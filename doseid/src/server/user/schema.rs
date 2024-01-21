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

#[derive(Serialize, Deserialize, Debug)]
struct UserGithub {
  login: String,
  id: i64,
  access_token: Option<String>,
  emails: Vec<UserGithubEmail>,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserGithubEmail {
  email: String,
  primary: bool,
  verified: bool,
  visibility: Option<String>,
}
