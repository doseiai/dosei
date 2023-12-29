mod cluster;
mod cron;
mod secret;

use anyhow::Context;
use sqlx::postgres::Postgres;
use sqlx::Pool;
use std::future::Future;
use std::sync::Arc;

use crate::config::Config;
use axum::{routing, Extension, Router};
use bollard::Docker;
use futures_util::TryFutureExt;
use tokio::net::TcpListener;
use tracing::{error, info};

pub async fn start_server(config: &'static Config) -> anyhow::Result<()> {
  check_docker_daemon_status().await;
  let pool = Pool::<Postgres>::connect(&config.database_url)
    .await
    .context("Failed to connect to Postgres")?;
  let shared_pool = Arc::new(pool);
  info!("Successfully connected to Postgres");
  cluster::start_cluster(config)?;
  cron::start_job_manager(config, Arc::clone(&shared_pool));
  let app = Router::new()
    .route("/envs/:owner_id", routing::post(secret::api_set_envs))
    .route(
      "/envs/:owner_id/:project_id",
      routing::post(secret::api_set_envs),
    )
    .route("/envs/:owner_id", routing::get(secret::api_get_envs))
    .route(
      "/envs/:owner_id/:project_id",
      routing::get(secret::api_get_envs),
    )
    .route("/cron-jobs", routing::post(cron::api_create_job))
    .route("/cron-jobs", routing::get(cron::api_get_cron_jobs))
    .layer(Extension(Arc::clone(&shared_pool)));
  let address = config.address.to_string();
  let listener = TcpListener::bind(&address)
    .await
    .context("Failed to start server")?;
  info!("Dosei running on http://{} (Press CTRL+C to quit", address);
  axum::serve(listener, app).await?;
  Ok(())
}

// TODO: Add this to some sort of healthcheck or periodically check, if this is dead, exit 1
async fn check_docker_daemon_status() {
  match Docker::connect_with_socket_defaults() {
    Ok(connection) => match connection.ping().await {
      Ok(_) => info!("Successfully connected to Docker Daemon"),
      Err(e) => {
        error!("Failed to ping Docker: {}", e);
        std::process::exit(1);
      }
    },
    Err(e) => {
      error!("Failed to connect to Docker: {}", e);
      std::process::exit(1);
    }
  };
}
