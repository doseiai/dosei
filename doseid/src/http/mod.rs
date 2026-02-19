mod health;
mod info;

use crate::config::Config;
use crate::deployment::Deployment;
use crate::session::Session;
use crate::{account, auth, deployment, ingress, node, service};
use anyhow::{anyhow, Context};
use axum::body::Body;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use axum::{middleware, Extension, Router};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::{debug, error, info};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::openapi::Components;
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
  modifiers(&SecurityAddon)
)]
struct ApiDoc;

pub struct Http;

impl Http {
  pub async fn start_server(
    config: &'static Config,
    shared_pool: &Arc<Pool<Postgres>>,
  ) -> anyhow::Result<()> {
    let mut api_doc = ApiDoc::openapi();

    let (public_router, public_api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
      .routes(routes!(health::health))
      .routes(routes!(info::info))
      .split_for_parts();
    api_doc.merge(public_api);

    let (private_router, private_api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
      .routes(routes!(account::route::api_user))
      .routes(routes!(account::route::api_list_user_ssh_key))
      .routes(routes!(service::route::api_list_services))
      .routes(routes!(deployment::route::api_deploy))
      .routes(routes!(deployment::route::api_list_service_deployments))
      .routes(routes!(ingress::route::api_list_service_ingresses))
      .routes(routes!(auth::route::login_ssh))
      .routes(routes!(auth::route::logout))
      .route_layer(middleware::from_fn(Session::middleware))
      .split_for_parts();
    api_doc.merge(private_api);

    // Internal routes (no auth, internal network only)
    let internal_router = Router::new()
      .route("/internal/nodes/register", axum::routing::post(node::route::register))
      .route("/internal/nodes/heartbeat", axum::routing::post(node::route::heartbeat))
      .route("/internal/nodes", axum::routing::get(node::route::list_nodes))
      .route("/internal/nodes/:id", axum::routing::delete(node::route::delete_node))
      .route("/internal/deploy", axum::routing::post(deployment::route::internal_deploy));

    let app = Router::new()
      .merge(public_router)
      .merge(private_router)
      .merge(internal_router)
      .merge(SwaggerUi::new("/docs").url("/openapi.json", api_doc))
      .layer(CorsLayer::permissive())
      .layer(Extension(Arc::clone(shared_pool)))
      .layer(Extension(config))
      .fallback(proxy_fallback);

    let listener = TcpListener::bind(&config.address())
      .await
      .context("Failed to start server")?;
    tokio::spawn(async move {
      info!(
        "DoseidD API running on http://{} (Press Ctrl+C to quit)",
        &config.address()
      );
      axum::serve(listener, app)
        .await
        .expect("Failed start DoseiD API");
    });
    signal::ctrl_c()
      .await
      .map_err(|err| anyhow!("Unable to listen for shutdown signal: {}", err))?;
    info!("Gracefully stopping... (Press Ctrl+C again to force)");
    Ok(())
  }
}

/// Fallback handler: routes unmatched requests by Host header to local containers.
/// Caddy forwards requests with the original Host header; this handler looks up
/// the ingress → deployment → local container port and proxies to it.
async fn proxy_fallback(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  req: Request,
) -> Result<Response, StatusCode> {
  let host = match req.headers().get("host") {
    Some(host_header) => host_header.to_str().unwrap_or_default().to_string(),
    None => return Err(StatusCode::NOT_FOUND),
  };
  debug!("Proxy fallback for host: {}", host);

  let path = req.uri().path().to_string();
  let path_query = req
    .uri()
    .path_and_query()
    .map(|v| v.as_str().to_string())
    .unwrap_or(path.clone());

  match Deployment::find_via_host(&host, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
  {
    None => Err(StatusCode::NOT_FOUND),
    Some(deployment) => match deployment.host_port {
      None => Err(StatusCode::NOT_FOUND),
      Some(host_port) => {
        let target_url = format!("http://127.0.0.1:{}{}", host_port, path_query);
        info!("Forwarding: {} -> {}", host, target_url);

        let client = reqwest::Client::new();
        let method = req.method().clone();
        let headers = req.headers().clone();
        let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
          .await
          .map_err(|_| StatusCode::BAD_REQUEST)?;

        let mut upstream_req = client.request(method, &target_url);
        for (key, value) in headers.iter() {
          upstream_req = upstream_req.header(key, value);
        }
        let upstream_resp = upstream_req
          .body(body_bytes)
          .send()
          .await
          .map_err(|e| {
            error!("Proxy request failed: {}", e);
            StatusCode::BAD_GATEWAY
          })?;

        let status = StatusCode::from_u16(upstream_resp.status().as_u16())
          .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let resp_headers = upstream_resp.headers().clone();
        let resp_body = upstream_resp.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;

        let mut response = Response::builder().status(status);
        for (key, value) in resp_headers.iter() {
          response = response.header(key, value);
        }
        let response = response
          .body(Body::from(resp_body))
          .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        deployment.update_last_accessed(&pg_pool);
        Ok(response)
      }
    },
  }
}

struct SecurityAddon;

impl Modify for SecurityAddon {
  fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
    if openapi.components.is_none() {
      openapi.components = Some(Components::new());
    }

    openapi.components.as_mut().unwrap().add_security_scheme(
      "Authentication",
      SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
    );
  }
}
