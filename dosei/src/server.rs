mod cluster;
mod cron;


use axum::{routing::get, Router};
use log::{info};
use crate::config::{Config };

pub async fn start_server(config: Config) {
  cluster::start_main_node();
  cron::start_job_manager();
  let mut address = format!("{}:{}", "0.0.0.0", "8844");
  if config.is_replica() {
    address = format!("{}:{}", "0.0.0.0", "8845");
  }
  let app = Router::new().route("/", get(|| async { "Hello, World!" }));
  info!("Dosei running on http://{} (Press CTRL+C to quit", &address);
  axum::Server::bind(&address.parse().unwrap()).serve(app.into_make_service()).await.unwrap();
}
