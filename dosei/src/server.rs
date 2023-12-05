mod cluster;
mod cron;

use axum::{routing::get, Router};
use log::{info};

pub async fn start_server() {
  let address = format!("{}:{}", "0.0.0.0", "8844");
  cluster::start_main_node();
  cron::start_job_manager();
  let app = Router::new().route("/", get(|| async { "Hello, World!" }));
  info!("Dosei running on http://{} (Press CTRL+C to quit", &address);
  axum::Server::bind(&address.parse().unwrap()).serve(app.into_make_service()).await.unwrap();
}
