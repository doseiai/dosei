use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CronJob {
  pub id: Uuid,
  pub schedule: String,
  pub entrypoint: String,
  pub owner_id: Uuid,
  pub project_id: Uuid,
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
  pub project_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Deployment {
  /// The unique identifier for the deployment, sourced from the Commit SHA.
  pub id: String,
  pub owner_id: Uuid,
  pub project_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Domain {
  pub id: Uuid,
  pub name: String,
  pub owner_id: Uuid,
  pub project_id: Option<Uuid>,
  pub storage_id: Option<Uuid>,
  pub deployment_id: Option<String>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl Domain {
  pub fn new(mut domain: Domain) -> Domain {
    domain.name = domain.name.to_lowercase();
    domain
  }
}
