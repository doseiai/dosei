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
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Mode {
  STANDALONE,
  CLUSTER,
}
