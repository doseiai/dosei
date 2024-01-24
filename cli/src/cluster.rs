use crate::config::Config;
use reqwest::Error;
use serde::{Deserialize, Serialize};

pub fn get_cluster_info(config: &'static Config) -> anyhow::Result<Info, Error> {
  let response = config
    .cluster_api_client()
    .expect("Client connection failed")
    .get(format!("{}/info", config.api_base_url))
    .send()?;
  if response.status().is_success() {
    let cluster_info = response.json::<Info>()?;
    return Ok(cluster_info);
  }
  let error = response.error_for_status_ref().err().unwrap();
  Err(error)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
  pub server: Server,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
  pub integration: Integration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Integration {
  pub github: Option<GithubIntegration>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubIntegration {
  pub app_name: String,
  pub app_id: String,
  pub client_id: String,
}
