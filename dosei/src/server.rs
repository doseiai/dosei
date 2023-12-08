mod cluster;
mod cron;

use axum::{routing::get, Router};
use log::{info};
use crate::config::{Config};

pub async fn start_server(config: &Config) {
  cluster::start_main_node(config);
  cron::start_job_manager();
  let mut address = format!("{}:{}", config.host, config.port);
  let app = Router::new().route("/", get(|| async { "Hello, World!" }));
  info!("Dosei running on http://{} (Press CTRL+C to quit", &address);
  axum::Server::bind(&address.parse().unwrap()).serve(app.into_make_service()).await.unwrap();
}
