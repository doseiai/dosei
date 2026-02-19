mod health;
mod info;

use crate::config::Config;
use crate::session::Session;
use crate::{account, auth, deployment, ingress, node, service};
use anyhow::{anyhow, Context};
use axum::{middleware, Extension, Router};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::info;
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
      .layer(Extension(config));

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

  /// Start a lightweight HTTP server for worker nodes.
  /// Only serves health check and internal deploy endpoint (no DB needed).
  pub async fn start_worker_server(config: &'static Config) -> anyhow::Result<()> {
    let app = Router::new()
      .route("/health", axum::routing::get(health::health))
      .route("/internal/deploy", axum::routing::post(deployment::route::internal_deploy))
      .layer(CorsLayer::permissive())
      .layer(Extension(config));

    let listener = TcpListener::bind(&config.address())
      .await
      .context("Failed to start worker server")?;
    tokio::spawn(async move {
      info!(
        "DoseiD Worker API running on http://{} (Press Ctrl+C to quit)",
        &config.address()
      );
      axum::serve(listener, app)
        .await
        .expect("Failed start DoseiD Worker API");
    });
    signal::ctrl_c()
      .await
      .map_err(|err| anyhow!("Unable to listen for shutdown signal: {}", err))?;
    info!("Gracefully stopping... (Press Ctrl+C again to force)");
    Ok(())
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
