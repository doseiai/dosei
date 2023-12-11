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

pub fn cron_job_mock() -> CronJob {
  CronJob {
    uuid: Uuid::new_v4(),
    schedule: "*/1 * * * *".to_string(),
    entrypoint: "bot.main:tweet".to_string(),
    owner_id: Uuid::new_v4(),
    deployment_id: Uuid::new_v4(),
    updated_at: Default::default(),
    created_at: Default::default(),
  }
}
