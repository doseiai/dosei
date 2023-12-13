mod cluster;
mod cron;

use sqlx::postgres::Postgres;
use sqlx::Pool;
use std::env;

use crate::config::Config;
use axum::{routing, Router};
use log::info;

pub async fn start_server(config: &'static Config) {
  let pool = Pool::<Postgres>::connect(&env::var("DATABASE_URL").unwrap())
    .await
    .unwrap();
  cluster::start_node(config);
  cron::start_job_manager(config, &pool);
  info!("Successfully connected to Postgres");
  let app = Router::new().route("/", routing::get(move || cron::get_cron_jobs(pool.clone())));
  let address = config.address.to_string();
  info!("Dosei running on http://{} (Press CTRL+C to quit", address);
  axum::Server::bind(&address.parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
}
