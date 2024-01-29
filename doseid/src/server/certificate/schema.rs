use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Certificate {
  pub id: Uuid,
  pub domain_name: String,
  pub certificate: String,
  pub private_key: String,
  pub expires_at: DateTime<Utc>,
  pub owner_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}
