use crate::config::Config;
use chrono::{DateTime, Utc};
use jsonwebtoken::{EncodingKey, Header};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
  pub id: Uuid,
  pub token: String,
  pub refresh_token: String,
  pub owner_id: Uuid,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionCredentials {
  pub id: Uuid,
  pub token: String,
  pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionToken {
  pub owner_id: Uuid,
}

impl Session {
  pub fn new(config: &'static Config, owner_id: Uuid) -> anyhow::Result<Session> {
    let token = jsonwebtoken::encode(
      &Header::default(),
      &SessionToken {
        owner_id: owner_id.to_owned(),
      },
      &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )?;
    Ok(Session {
      id: Uuid::new_v4(),
      token,
      refresh_token: thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect(),
      owner_id,
      updated_at: Utc::now(),
      created_at: Utc::now(),
    })
  }
  pub fn session_credentials(&self) -> SessionCredentials {
    SessionCredentials {
      id: self.id,
      token: self.token.clone(),
      refresh_token: self.refresh_token.clone(),
    }
  }
}
