use crate::server::integration::github::schema::UserGithub;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
  pub id: Uuid,
  pub username: String,
  pub name: Option<String>,
  pub email: String,
  pub github: Option<UserGithub>,
  pub gitlab: Option<Value>,
  pub bitbucket: Option<Value>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}
