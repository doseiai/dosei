use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
#[sqlx(type_name = "status", rename_all = "lowercase")]
pub enum DeploymentStatus {
  Queued,
  Building,
  Error,
  Canceled,
  Ready,
}