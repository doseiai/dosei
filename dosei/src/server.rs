mod cluster;
mod cron;
mod secret;

use sqlx::postgres::Postgres;
use sqlx::Pool;
use std::env;
use std::sync::Arc;

use crate::config::Config;
use axum::{routing, Extension, Router};
use log::info;
use tokio::net::TcpListener;

pub async fn start_server(config: &'static Config) -> anyhow::Result<()> {
  let pool = Pool::<Postgres>::connect(&env::var("DATABASE_URL")?).await?;
  let shared_pool = Arc::new(pool);
  info!("Successfully connected to Postgres");
  cluster::start_node(config);
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
  let listener = TcpListener::bind(&address).await?;
  info!("Dosei running on http://{} (Press CTRL+C to quit", address);
  axum::serve(listener, app).await?;
  Ok(())
}
