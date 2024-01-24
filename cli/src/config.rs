use home::home_dir;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use sysinfo::System;
use uuid::Uuid;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
// TODO: Get this from the API directly
pub const GITHUB_CLIENT_ID: &str = "Iv1.261badedf2d43fd5";

#[derive(Debug)]
pub struct Config {
  pub api_base_url: String,
  token: Option<String>,
  credentials_path: PathBuf,
}

impl Config {
  pub fn new() -> anyhow::Result<Config> {
    Ok(Config {
      api_base_url: env::var("API_BASE_URL").unwrap_or_else(|_| "https://api.dosei.ai".to_string()),
      token: env::var("DOSEI_TOKEN").ok(),
      credentials_path: home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".dosei/credentials.json"),
    })
  }

  pub fn store_token_from_session(&self, session: &SessionCredentials) -> anyhow::Result<()> {
    if let Some(parent) = self.credentials_path.parent() {
      fs::create_dir_all(parent).expect("Failed to create directories");
    }

    let mut file = File::create(&self.credentials_path).expect("Failed to create file");
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
    fs::remove_file(&self.credentials_path).expect("Failed to remove file");
    Ok(())
  }

  pub fn cluster_api_client(&self) -> Result<Client, reqwest::Error> {
    let mut headers = HeaderMap::new();
    let user_agent_value = format!(
      "Dosei/{} ({} {}) CLI",
      VERSION,
      System::name().unwrap(),
      System::os_version().unwrap()
    );
    headers.insert(
      reqwest::header::USER_AGENT,
      reqwest::header::HeaderValue::from_str(&user_agent_value).unwrap(),
    );
    let client = Client::builder().default_headers(headers).build()?;
    Ok(client)
  }

  pub fn session(&self) -> Option<SessionCredentials> {
    if !self.credentials_path.exists() {
      return None;
    }

    let mut file = File::open(&self.credentials_path).expect("Failed to open file");
    let mut contents = String::new();
    file
      .read_to_string(&mut contents)
      .expect("Failed to read file");

    Some(serde_json::from_str(&contents).expect("Failed to deserialize"))
  }

  pub fn bearer_token(&self) -> String {
    self.session_token().expect(
      "
      To get started with Dosei CLI, please run: dosei login\n\
      Alternatively, populate the DOSEI_TOKEN environment variable with a Dosei API authentication token\
      "
    )
  }

  pub fn session_token(&self) -> Option<String> {
    if let Some(token) = &self.token {
      return Some(token.clone());
    }

    if !self.credentials_path.exists() {
      return None;
    }

    let mut file = File::open(&self.credentials_path).expect("Failed to open file");
    let mut contents = String::new();
    file
      .read_to_string(&mut contents)
      .expect("Failed to read file");

    let credentials: SessionCredentials =
      serde_json::from_str(&contents).expect("Failed to deserialize");
    Some(credentials.token)
  }
}

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
