use crate::config::{Address, Config, VERSION};
use crate::server::cluster::CLUSTER_INFO;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub async fn api_info(config: Extension<&'static Config>) -> Result<Json<Info>, StatusCode> {
  let cluster_info = Arc::clone(&CLUSTER_INFO);

  Ok(Json(Info {
    server: Server {
      id: config.node_info.id,
      mode: if config.is_primary() && cluster_info.lock().await.replicas.is_empty() {
        Mode::STANDALONE
      } else {
        Mode::CLUSTER
      },
      address: config.address.clone(),
      version: VERSION.to_string(),
      integration: Integration {
        github: config
          .github_integration
          .as_ref()
          .map(|github| GithubIntegration {
            app_name: github.app_name.clone(),
            app_id: github.app_id.clone(),
            client_id: github.client_id.clone(),
          }),
      },
    },
  }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Info {
  server: Server,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
  id: Uuid,
  mode: Mode,
  address: Address,
  version: String,
  integration: Integration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Integration {
  github: Option<GithubIntegration>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubIntegration {
  pub app_name: String,
  pub app_id: String,
  pub client_id: String,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Mode {
  STANDALONE,
  CLUSTER,
}
