use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CronJob {
  pub id: Uuid,
  pub schedule: String,
  pub entrypoint: String,
  pub owner_id: Uuid,
  pub deployment_id: String,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
  pub id: Uuid,
  pub cron_job_id: Uuid,
  pub exit_code: u8,
  pub logs: Vec<String>,
  pub entrypoint: String,
  pub owner_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Secret {
  pub id: Uuid,
  pub name: String,
  pub value: String,
  pub owner_id: Uuid,
  pub project_id: Option<Uuid>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}
