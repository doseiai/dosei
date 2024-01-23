use crate::server::integration::github::schema::UserGithub;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug)]
pub struct User {
  pub id: Uuid,
  pub username: String,
  pub name: Option<String>,
  pub email: String,
  pub github: Option<Value>,
  pub gitlab: Option<Value>,
  pub bitbucket: Option<Value>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl User {
  pub fn deserialize_github(&self) -> Result<Option<UserGithub>, serde_json::Error> {
    if let Some(github_json) = &self.github {
      let github_data: UserGithub = serde_json::from_value(github_json.clone())?;
      Ok(Some(github_data))
    } else {
      Ok(None)
    }
  }
}
