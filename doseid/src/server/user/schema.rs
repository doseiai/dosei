use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct User {
  pub id: Uuid,
  pub name: String,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}
