use home::home_dir;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{PathBuf};
use uuid::Uuid;

// TODO: Get this from the API directly
pub const GITHUB_CLIENT_ID: &str = "Iv1.0d2388105db85287";

#[derive(Debug)]
pub struct Config {
  api_base_url: String,
  token: Option<String>,
}

impl Config {
  pub fn new() -> anyhow::Result<Config> {
    Ok(Config {
      api_base_url: env::var("API_BASE_URL").unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_string()),
      token: env::var("DOSEI_TOKEN").ok(),
    })
  }

  pub fn store_token_from_session(&self, session: &SessionCredentials) -> anyhow::Result<()> {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(CREDENTIALS_PATH);
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).expect("Failed to create directories");
    }

    let mut file = File::create(path).expect("Failed to create file");
    file
      .write_all(
        serde_json::to_string_pretty(session)
          .expect("Failed to serialize")
          .as_bytes(),
      )
      .expect("Failed to write to file");
    Ok(())
  }

  pub fn remove_stored_credentials(&self) -> anyhow::Result<()> {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(CREDENTIALS_PATH);
    fs::remove_file(path).expect("Failed to remove file");
    Ok(())
  }

  pub fn session(&self) -> Option<SessionCredentials> {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(CREDENTIALS_PATH);
    if !path.exists() {
      return None;
    }

    let mut file = File::open(path).expect("Failed to open file");
    let mut contents = String::new();
    file
      .read_to_string(&mut contents)
      .expect("Failed to read file");

    Some(serde_json::from_str(&contents).expect("Failed to deserialize"))
  }

  pub fn session_token(&self) -> Option<String> {
    if let Some(token) = &self.token {
      return Some(token.clone());
    }

    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(CREDENTIALS_PATH);
    if !path.exists() {
      return None;
    }

    let mut file = File::open(path).expect("Failed to open file");
    let mut contents = String::new();
    file
      .read_to_string(&mut contents)
      .expect("Failed to read file");

    let credentials: SessionCredentials =
      serde_json::from_str(&contents).expect("Failed to deserialize");
    Some(credentials.token)
  }
}

const CREDENTIALS_PATH: &str = ".dosei/credentials.json";
const DEFAULT_API_BASE_URL: &str = "https://api.dosei.ai";

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionCredentials {
  pub id: Uuid,
  pub token: String,
  pub refresh_token: String,
}

#[cfg(test)]
mod tests {
  use crate::config::SessionCredentials;
  use crate::test::CONFIG;
  use uuid::Uuid;

  #[test]
  fn test_create_session_and_delete() {
    let result = CONFIG.store_token_from_session(&SessionCredentials {
      id: Uuid::default(),
      token: "test".to_string(),
      refresh_token: "test".to_string(),
    });
    assert!(result.is_ok());

    assert!(CONFIG.session().is_some());

    assert!(CONFIG.session_token().is_some());

    let result = CONFIG.remove_stored_credentials();
    assert!(result.is_ok());
  }
}
