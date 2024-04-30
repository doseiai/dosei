mod default;

use anyhow::anyhow;
use home::home_dir;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{env, fs};
use sysinfo::System;
use uuid::Uuid;

#[derive(Debug)]
pub struct Config {
  pub api_base_url: String,
  credentials_path: PathBuf,
}

impl Config {
  pub fn new() -> anyhow::Result<Config> {
    let home = home_dir().ok_or(anyhow!("Home directory not found"))?;
    Ok(Config {
      api_base_url: env::var("DOSEI_API_BASE_URL").unwrap_or(default::API_BASE_URL.to_string()),
      credentials_path: home.join(".dosei/credentials.json"),
    })
  }

  pub fn api_client(&self) -> Result<Client, reqwest::Error> {
    let mut headers = HeaderMap::new();
    let user_agent_value = format!(
      "Dosei/{} ({} {}) CLI",
      env!("CARGO_PKG_VERSION"),
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
    self
      .session_token()
      .expect("To get started with Dosei CLI, please run: dosei login")
  }

  pub fn session_token(&self) -> Option<String> {
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionCredentials {
  pub id: Uuid,
  pub token: String,
  pub refresh_token: String,
}
