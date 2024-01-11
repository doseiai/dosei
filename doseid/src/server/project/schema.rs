use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
#[sqlx(type_name = "git_source", rename_all = "lowercase")]
pub enum GitSource {
  Github,
  Gitlab,
  Bitbucket,
}
