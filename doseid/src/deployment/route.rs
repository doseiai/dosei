use crate::config::{Config, NodeMode};
use crate::deployment::Deployment;
use crate::ingress::Ingress;
use crate::node::Node;
use crate::service::Service;
use crate::session::AuthSession;
use crate::caddy;
use axum::extract::{Multipart, Path};
use axum::http::StatusCode;
use axum::{Extension, Json};
use dosei_schema::app::App;
use dosei_schema::cluster::ClusterInit;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tracing::{error, info, warn};
use utoipa::gen::serde_json::{json, Value};
use uuid::Uuid;

const TAG: &str = "deployment";

#[utoipa::path(
  get,
  path = "/service/{service_id}/deployment",
  params(
    ("service_id" = String, Path, description = "Service ID"),
  ),
  responses(
        (status = StatusCode::OK, body = Vec<Deployment>),
  ),
  security(
      ("Authentication" = [])
  ),
  tag = TAG
)]
pub async fn api_list_service_deployments(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Extension(AuthSession(session)): Extension<AuthSession>,
  Path(service_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Vec<Deployment>>), StatusCode> {
  let service = Service::get_by_id(service_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
  if service.owner_id != session.account_id {
    return Err(StatusCode::NOT_FOUND);
  }
  let deployments = Deployment::get_by_service_id(service_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok((StatusCode::OK, Json(deployments)))
}

#[utoipa::path(
  post,
  path = "/deploy",
  responses(
        (status = StatusCode::OK, body = Value),
  ),
  security(
      ("Authentication" = [])
  ),
  tag = TAG
)]
pub async fn api_deploy(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Extension(config): Extension<&'static Config>,
  Extension(AuthSession(session)): Extension<AuthSession>,
  mut multipart: Multipart,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
  let mut app = String::new();
  let mut hash = String::new();
  let mut file_data = Vec::new();
  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?
  {
    if let Some(name) = field.name() {
      match name {
        "app" => {
          app = String::from_utf8(
            field
              .bytes()
              .await
              .map_err(|_| StatusCode::BAD_REQUEST)?
              .to_vec(),
          )
          .map_err(|_| StatusCode::BAD_REQUEST)?;
        }
        "file" => {
          file_data = field
            .bytes()
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?
            .to_vec();
        }
        "hash" => {
          hash = String::from_utf8(
            field
              .bytes()
              .await
              .map_err(|_| StatusCode::BAD_REQUEST)?
              .to_vec(),
          )
          .map_err(|_| StatusCode::BAD_REQUEST)?;
        }
        _ => {} // Ignore other fields
      }
    }
  }

  let app = App::from_string(&app).map_err(|_| StatusCode::BAD_REQUEST)?;

  let service = match Service::new(&app.name, session.account_id, &pg_pool).await {
    Ok(service) => service,
    Err(_) => Service::get_by_name(app.name.clone(), &pg_pool)
      .await
      .map_err(|e| {
        error!("Failed to get service '{}': {}", app.name, e);
        StatusCode::INTERNAL_SERVER_ERROR
      })?
      .ok_or_else(|| {
        error!("Service '{}' not found", app.name);
        StatusCode::INTERNAL_SERVER_ERROR
      })?,
  };

  // Stop, remove, and delete previous deployments
  let previous_deployments = Deployment::get_by_service_id(service.id, &pg_pool)
    .await
    .unwrap_or_default();
  for prev in &previous_deployments {
    if let Err(e) = prev.stop().await {
      warn!("Failed to stop previous deployment {}: {}", prev.id, e);
    }
    if let Err(e) = prev.remove().await {
      warn!("Failed to remove previous deployment {}: {}", prev.id, e);
    }
    if let Err(e) = prev.delete(&pg_pool).await {
      warn!("Failed to delete previous deployment record {}: {}", prev.id, e);
    }
  }

  let deployment = Deployment::new(service.id, service.owner_id, app.port, None, None, &pg_pool)
    .await
    .map_err(|e| {
      error!("Failed to create deployment: {}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let build_logs = deployment
    .build(&file_data)
    .await
    .map_err(|e| {
      error!("Failed to build deployment {}: {}", deployment.id, e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  info!("Build completed for deployment {}", deployment.id);

  deployment
    .start(None, app.env.as_ref())
    .await
    .map_err(|e| {
      error!("Failed to start deployment {}: {}", deployment.id, e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  info!("Started deployment {}", deployment.id);

  if let Some(domains) = &app.domains {
    if !domains.is_empty() {
      let domain = domains.first().unwrap();
      if ClusterInit::validate_domain(domain) {
        let _ = Ingress::new(domain.clone(), service.id, service.owner_id, &pg_pool).await;
      }
    }
  }

  // Sync Caddy config so new container ports are routed
  caddy::trigger_sync(Arc::clone(&pg_pool));

  // Forward deploy to worker nodes
  if config.mode == NodeMode::Main {
    let nodes = Node::list_active(&pg_pool)
      .await
      .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let app_json = serde_json::to_string(&app).unwrap_or_default();

    for node in nodes.iter().filter(|n| !n.is_main) {
      let worker_url = format!("http://{}:{}/internal/deploy", node.ip, node.port);
      info!("Forwarding deploy to worker node: {}", worker_url);

      let client = reqwest::Client::new();
      let form = reqwest::multipart::Form::new()
        .part(
          "file",
          reqwest::multipart::Part::bytes(file_data.clone()).file_name("output.tar.gz"),
        )
        .text("app", app_json.clone())
        .text("hash", hash.clone());

      match client.post(&worker_url).multipart(form).send().await {
        Ok(resp) if resp.status().is_success() => {
          info!("Worker {} deployed successfully", node.ip);
        }
        Ok(resp) => {
          warn!("Worker {} deploy failed: {}", node.ip, resp.status());
        }
        Err(e) => {
          warn!("Worker {} deploy failed: {}", node.ip, e);
        }
      }
    }
  }

  Ok((StatusCode::OK, Json(json!({
    "deployment_id": deployment.id,
    "build_logs": build_logs,
  }))))
}

/// Internal deploy endpoint for worker nodes (no auth, no DB, called by main node).
/// Builds the image and starts the container locally.
pub async fn internal_deploy(
  mut multipart: Multipart,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
  let mut app_str = String::new();
  let mut file_data = Vec::new();
  while let Some(field) = multipart
    .next_field()
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?
  {
    if let Some(name) = field.name() {
      match name {
        "app" => {
          app_str = String::from_utf8(
            field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec(),
          )
          .map_err(|_| StatusCode::BAD_REQUEST)?;
        }
        "file" => {
          file_data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec();
        }
        _ => {}
      }
    }
  }

  let app = App::from_string(&app_str).map_err(|_| StatusCode::BAD_REQUEST)?;

  // Worker doesn't use DB — create a temporary Deployment to build and run
  let deployment = Deployment {
    id: Uuid::new_v4(),
    service_id: Uuid::new_v4(),
    owner_id: Uuid::nil(),
    host_port: app.port.map(|p| Deployment::find_available_host_port().unwrap_or(p)),
    container_port: app.port,
    last_accessed_at: None,
    updated_at: chrono::Utc::now(),
    created_at: chrono::Utc::now(),
    node_id: None,
  };

  deployment
    .build(&file_data)
    .await
    .map_err(|e| {
      error!("Internal deploy build failed for {}: {}", app.name, e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  deployment
    .start(None, app.env.as_ref())
    .await
    .map_err(|e| {
      error!("Internal deploy start failed for {}: {}", app.name, e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  info!("Internal deploy completed for app: {}", app.name);
  Ok((StatusCode::OK, Json(json!({}))))
}
