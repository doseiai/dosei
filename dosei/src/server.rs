mod cluster;
mod cron;

use sqlx::Pool;
use sqlx::postgres::Postgres;

use axum::{routing, Router};
use log::{info};
use crate::config;
use crate::config::{Config};

pub async fn start_server(config: &Config) {
  cluster::start_node(config);
  cron::start_job_manager(config);
  let pool = Pool::<Postgres>::connect(&config::postgres_url()).await.unwrap();
  info!("Successfully connected to Postgres");
  let app = Router::new().route("/", routing::get(|| async { "Hello, World!" }));
  let address = config.address.to_string();
  info!("Dosei running on http://{} (Press CTRL+C to quit", address);
  axum::Server::bind(&address.parse().unwrap()).serve(app.into_make_service()).await.unwrap();
}
