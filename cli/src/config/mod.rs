use crate::file::expand_tilde;
use crate::ssh::CliSSHBearerPayload;
use base64::engine::general_purpose;
use base64::Engine;
use reqwest::blocking::Client;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{env, fs, io};
use sysinfo::System;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClusterConfig {
  pub id: Option<Uuid>,
  pub username: String,
  pub ssh_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
  pub clusters: Option<HashMap<String, ClusterConfig>>,
  pub default_cluster: Option<String>,
}

impl Config {
  fn new() -> Self {
    Config {
      clusters: Some(HashMap::new()),
      default_cluster: None,
    }
  }

  pub fn load() -> io::Result<Self> {
    Self::create_dir()?;

    match File::open(Self::path()) {
      Ok(mut file) => {
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        match serde_json::from_str(&contents) {
          Ok(config) => Ok(config),
          Err(_) => Ok(Self::new()),
        }
      }
      Err(_) => Ok(Self::default()),
    }
  }

  pub fn save(&self) -> io::Result<()> {
    Self::create_dir()?;

    // Serialize to JSON
    let json = serde_json::to_string_pretty(self)?;

    // Write to file
    let mut file = File::create(Self::path())?;
    file.write_all(json.as_bytes())?;

    Ok(())
  }

  pub fn add_cluster(&mut self, name: String, config: ClusterConfig) -> Option<ClusterConfig> {
    // Initialize clusters if None
    if self.clusters.is_none() {
      self.clusters = Some(HashMap::new());
    }

    if let Some(clusters) = &mut self.clusters {
      clusters.insert(name, config)
    } else {
      None
    }
  }

  #[allow(dead_code)]
  pub fn remove_cluster(&mut self, name: &str) -> Option<ClusterConfig> {
    if let Some(clusters) = &mut self.clusters {
      clusters.remove(name)
    } else {
      None
    }
  }

  pub fn get_default_cluster(&self) -> Option<HashMap<String, ClusterConfig>> {
    if let Some(default_cluster) = &self.default_cluster {
      Some(HashMap::from([(
        default_cluster.clone(),
        self.get_cluster(default_cluster)?.clone(),
      )]))
    } else {
      None
    }
  }

  pub fn get_cluster(&self, name: &str) -> Option<&ClusterConfig> {
    if let Some(clusters) = &self.clusters {
      clusters.get(name)
    } else {
      None
    }
  }

  pub fn list_clusters(&self) -> HashMap<String, ClusterConfig> {
    self.clusters.clone().unwrap_or_default()
  }

  #[allow(dead_code)]
  pub fn update_cluster(&mut self, name: &str, config: ClusterConfig) -> Option<ClusterConfig> {
    if let Some(clusters) = &mut self.clusters {
      if clusters.contains_key(name) {
        clusters.insert(name.to_string(), config)
      } else {
        None
      }
    } else {
      None
    }
  }

  fn create_dir() -> io::Result<()> {
    if let Some(parent) = Self::path().parent() {
      fs::create_dir_all(parent)?;
    }
    Ok(())
  }

  fn path() -> PathBuf {
    expand_tilde("~/.dosei/config.json")
  }
}

pub struct ApiClient;

impl ApiClient {
  pub fn default() -> Result<Client, reqwest::Error> {
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

  pub fn bearer_ssh_token(ssh_key_path: Option<PathBuf>) -> anyhow::Result<String> {
    let token = CliSSHBearerPayload::new(ssh_key_path)?;
    Ok(format!("ssh:{}", token.to_base64()?))
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionCredentials {
  pub id: Uuid,
  pub token: String,
  pub refresh_token: String,
}

impl SessionCredentials {
  pub fn to_base64(&self) -> anyhow::Result<String> {
    let json = serde_json::to_string(self)?;
    Ok(general_purpose::STANDARD.encode(json.as_bytes()))
  }
}
