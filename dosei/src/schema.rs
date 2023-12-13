use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct CronJob {
  pub uuid: Uuid,
  pub schedule: String,
  pub entrypoint: String,
  pub owner_id: Uuid,
  pub deployment_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}
