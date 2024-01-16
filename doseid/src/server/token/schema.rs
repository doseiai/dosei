use chrono::{DateTime, Duration, Utc};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
  pub id: Uuid,
  pub name: String,
  pub value: String,
  pub owner_id: Uuid,
  pub expires_in: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl Token {
  pub fn new(
    name: String,
    days_until_expiration: i32,
    owner_id: Uuid,
  ) -> Result<Token, Box<dyn Error>> {
    if days_until_expiration < -1 || days_until_expiration == 0 {
      return Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "days_until_expiration must number of days or -1 for non expiration",
      )));
    }
    let now = Utc::now();
    Ok(Token {
      id: Uuid::new_v4(),
      name,
      value: thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect(),
      owner_id,
      expires_in: if days_until_expiration == -1 {
        DateTime::<Utc>::MAX_UTC
      } else {
        now + Duration::days(i64::from(days_until_expiration))
      },
      updated_at: now,
      created_at: now,
    })
  }
}

#[cfg(test)]
mod tests {
  use crate::server::token::schema::Token;
  use uuid::Uuid;

  #[test]
  fn test_tokens() {
    for n in [-1, 1, 7, 30, 60, 180, 365] {
      let token = Token::new("example".to_string(), n, Uuid::default());
      assert!(token.is_ok());
    }
    let result = Token::new("wrong_example".to_string(), 0, Uuid::default());
    assert!(result.is_err());

    let result = Token::new("wrong_example".to_string(), -2, Uuid::default());
    assert!(result.is_err())
  }
}
