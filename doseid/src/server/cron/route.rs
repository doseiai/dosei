use crate::server::cron::get_cron_jobs;
use crate::server::cron::schema::CronJob;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

pub async fn api_create_job(
  pool: Extension<Arc<Pool<Postgres>>>,
  Json(body): Json<CreateJobBody>,
) -> Result<Json<CronJob>, StatusCode> {
  let cron_job = CronJob {
    id: Uuid::new_v4(),
    schedule: body.schedule,
    entrypoint: body.entrypoint,
    owner_id: body.owner_id,
    project_id: body.project_id,
    deployment_id: body.deployment_id,
    updated_at: Default::default(),
    created_at: Default::default(),
  };
  match sqlx::query_as!(
    CronJob,
    "
    INSERT INTO cron_job (id, schedule, entrypoint, owner_id, project_id, deployment_id, updated_at, created_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    RETURNING *
    ",
    cron_job.id,
    cron_job.schedule,
    cron_job.entrypoint,
    cron_job.owner_id,
    cron_job.project_id,
    cron_job.deployment_id,
    cron_job.updated_at,
    cron_job.created_at
  ).fetch_one(&**pool).await
  {
    Ok(recs) => Ok(Json(recs)),
    Err(err) => {
      error!("Error in creating job: {:?}", err);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

pub async fn api_get_cron_jobs(
  pool: Extension<Arc<Pool<Postgres>>>,
) -> Result<Json<Vec<CronJob>>, StatusCode> {
  match get_cron_jobs(pool.0).await {
    Ok(result) => Ok(result),
    Err(err) => {
      error!("Error in reading job: {:?}", err);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

#[derive(Deserialize)]
pub struct CreateJobBody {
  schedule: String,
  entrypoint: String,
  owner_id: Uuid,
  project_id: Uuid,
  deployment_id: String,
}
