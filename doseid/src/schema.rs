use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
#[sqlx(type_name = "git_source", rename_all = "lowercase")]
pub enum GitSource {
  Github,
  Gitlab,
  Bitbucket,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
  pub id: Uuid,
  pub name: String,
  pub owner_id: Uuid,
  pub git_source: GitSource,
  pub git_source_metadata: Value,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
#[sqlx(type_name = "status", rename_all = "lowercase")]
pub enum DeploymentStatus {
  Queued,
  Building,
  Error,
  Canceled,
  Ready,
}

pub struct Deployment {
  pub id: Uuid,
  pub commit_id: String,
  pub commit_metadata: Value,
  pub project_id: Uuid,
  pub owner_id: Uuid,
  pub status: DeploymentStatus,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
#[sqlx(type_name = "service_type", rename_all = "lowercase")]
pub enum ServiceType {
  Queued,
  Building,
  Error,
  Canceled,
  Ready,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Domain {
  pub id: Uuid,
  pub name: String,
  pub service_type: ServiceType,
  pub project_id: Option<Uuid>,
  pub storage_id: Option<Uuid>,
  pub deployment_id: Option<String>,
  pub owner_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl Domain {
  pub fn new(mut domain: Domain) -> Domain {
    domain.name = domain.name.to_lowercase();
    domain
  }
}
