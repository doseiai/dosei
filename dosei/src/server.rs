mod cluster;
mod cron;

use std::env;
use sqlx::Pool;
use sqlx::postgres::Postgres;

use axum::{routing, Router, Json};
use log::{info};
use crate::{schema};
use crate::config::{Config};
use crate::schema::CronJob;

async fn get_cron_jobs(pool: Pool<Postgres>) -> Json<Vec<CronJob>> {
  let recs = sqlx::query_as!(schema::CronJob, "SELECT * from cron_jobs")
    .fetch_all(&pool).await.unwrap();
  Json(recs)
}

pub async fn start_server(config: &Config) {
  cluster::start_node(config);
  cron::start_job_manager(config);
  let pool = Pool::<Postgres>::connect(&env::var("DATABASE_URL").unwrap()).await.unwrap();
  let cron_job = schema::cron_job_mock();
  let rec = sqlx::query!(
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
  let app = Router::new().route("/", routing::get(move || get_cron_jobs(pool.clone())));
  let address = config.address.to_string();
  info!("Dosei running on http://{} (Press CTRL+C to quit", address);
  axum::Server::bind(&address.parse().unwrap()).serve(app.into_make_service()).await.unwrap();
}
