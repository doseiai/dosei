mod cluster;
mod cron;

use sqlx::postgres::Postgres;
use sqlx::Pool;
use std::env;

use crate::config::Config;
use crate::schema;
use axum::{routing, Router};
use log::info;

pub async fn start_server(config: &'static Config) {
  let pool = Pool::<Postgres>::connect(&env::var("DATABASE_URL").unwrap())
    .await
    .unwrap();
  cluster::start_node(&config);
  cron::start_job_manager(&config, &pool);
  let cron_job = schema::cron_job_mock();
  sqlx::query!(
    r#"
    INSERT INTO cron_jobs (uuid, schedule, entrypoint, owner_id, deployment_id, updated_at, created_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    RETURNING uuid
    "#,
    cron_job.uuid,
    cron_job.schedule,
    cron_job.entrypoint,
    cron_job.owner_id,
    cron_job.deployment_id,
    cron_job.updated_at,
    cron_job.created_at
  ).fetch_one(&pool).await.unwrap();
  info!("Successfully connected to Postgres");
  let app = Router::new().route("/", routing::get(move || cron::get_cron_jobs(pool.clone())));
  let address = config.address.to_string();
  info!("Dosei running on http://{} (Press CTRL+C to quit", address);
  axum::Server::bind(&address.parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
}
