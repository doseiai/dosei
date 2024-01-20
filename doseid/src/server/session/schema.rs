use crate::config::Config;
use jsonwebtoken::{EncodingKey, Header};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
  pub owner_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionCredentials {
  pub id: Uuid,
  token: String,
  refresh_token: String,
}

impl SessionCredentials {
  pub fn new(config: &'static Config, owner_id: Uuid) -> anyhow::Result<SessionCredentials> {
    let token = jsonwebtoken::encode(
      &Header::default(),
      &Session {
        owner_id: owner_id.to_owned(),
      },
      &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    )?;
    Ok(SessionCredentials {
      id: Uuid::new_v4(),
      token,
      refresh_token: thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect(),
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::server::session::schema::SessionCredentials;
  use crate::test::CONFIG;
  use uuid::Uuid;

  #[test]
  fn test_new_session() {
    let session = SessionCredentials::new(&CONFIG, Uuid::new_v4());
    assert!(session.is_ok())
  }
}
