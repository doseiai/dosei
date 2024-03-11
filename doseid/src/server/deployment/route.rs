use crate::config::Config;
use crate::deployment::app::import_dosei_app;
use crate::docker::build_image_raw;
use crate::server::deployment::schema::{Deployment, DeploymentStatus};
use crate::server::session::validate_session;
use crate::server::user::get_user;
use crate::util::extract_tar_gz_from_memory;
use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use bollard::container::{CreateContainerOptions, StartContainerOptions};
use bollard::models::{HostConfig, PortBinding, PortMap};
use bollard::Docker;
use chrono::Utc;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;
use tracing::error;
use tracing_subscriber::fmt::format::json;
use uuid::Uuid;

pub async fn api_deploy(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  mut multipart: Multipart,
) -> Result<Response, StatusCode> {
  let session = validate_session(Arc::clone(&pool), &config, headers).await?;
  let user = get_user(session.owner_id, Arc::clone(&pool))
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  let deployment = Deployment {
    id: Uuid::new_v4(),
    commit_id: "non_tracked".to_string(),
    commit_metadata: json!({}),
    project_id: Uuid::new_v4(),
    owner_id: session.owner_id,
    status: DeploymentStatus::Building,
    build_logs: json!({}),
    updated_at: Utc::now(),
    created_at: Utc::now(),
  };

  sqlx::query_as!(
      Deployment,
      "
      INSERT INTO deployment (id, commit_id, commit_metadata, project_id, owner_id, status, build_logs, updated_at, created_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
      ",
      deployment.id,
      deployment.commit_id,
      deployment.commit_metadata,
      deployment.project_id,
      deployment.owner_id,
      deployment.status as DeploymentStatus,
      deployment.build_logs,
      deployment.updated_at,
      deployment.created_at,
    ).execute(&**pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  let mut combined_data = Vec::new();
  while let Some(field) = multipart.next_field().await.unwrap() {
    let data = field.bytes().await.unwrap();
    combined_data.extend(data.clone());
  }
  let image_tag = format!("{}/{}", Uuid::new_v4(), Uuid::new_v4());

  let build_logs = build_image_raw(&image_tag, &combined_data)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  let temp_dir = tempdir().expect("Failed to create a temp dir");
  let temp_path = temp_dir.path();
  extract_tar_gz_from_memory(&combined_data, temp_path)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  let app = import_dosei_app(&image_tag, temp_path)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  // Create the exposed port key
  let exposed_port = format!("{}/tcp", &app.port);

  // Initialize exposed ports map
  let empty = HashMap::new();
  let mut exposed_ports = HashMap::new();
  exposed_ports.insert(exposed_port.as_str(), empty);

  // Initialize port bindings
  let port_binding = vec![PortBinding {
    host_ip: Some("0.0.0.0".to_string()),
    host_port: Some(app.port.to_string()),
  }];
  let mut port_map = PortMap::new();
  port_map.insert(format!("{}/tcp", &app.port), Some(port_binding));

  let host_config = HostConfig {
    port_bindings: Some(port_map),
    ..Default::default()
  };

  let config = bollard::container::Config {
    image: Some(image_tag.as_str()),
    cmd: Some(app.run.split_whitespace().collect()),
    exposed_ports: Some(exposed_ports),
    host_config: Some(host_config),
    tty: Some(true),
    ..Default::default()
  };

  let docker = Docker::connect_with_socket_defaults().unwrap();
  let container = docker
    .create_container(None::<CreateContainerOptions<String>>, config)
    .await
    .unwrap();

  if let Err(e) = docker
    .start_container(&container.id, None::<StartContainerOptions<String>>)
    .await
  {
    error!("Error starting container: {:?}", e)
  }
  sqlx::query_as!(
    Deployment,
    "UPDATE deployment SET status = $1, updated_at = $2, build_logs = $3  WHERE id = $4::uuid",
    DeploymentStatus::Ready as DeploymentStatus,
    Utc::now(),
    json!(build_logs),
    deployment.id,
  )
  .execute(&**pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok(StatusCode::CREATED.into_response())
}
