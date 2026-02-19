mod account;
mod auth;
mod caddy;
mod cluster;
mod config;
mod container;
mod deployment;
mod http;
mod ingress;
mod job;
mod node;
mod service;
mod session;

use crate::caddy::trigger_sync;
use crate::cluster::DaemonClusterInit;
use crate::config::{Config, NodeMode};
use crate::container::Container;
use crate::http::Http;
use crate::job::Job;
use crate::node::{start_heartbeat_task, start_stale_node_cleanup, Node};
use anyhow::Context;
use doseid::PluginManager;
use sqlx::{Pool, Postgres};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));

  Container::check_docker_daemon_status().await;

  let pg_pool = Pool::<Postgres>::connect(&config.database_url)
    .await
    .context("Failed to connect to Postgres")?;
  sqlx::migrate!().run(&pg_pool).await?;
  let shared_pool = Arc::new(pg_pool);

  let cluster = DaemonClusterInit::new()
    .await
    .context("Cluster creation failed")?;
  cluster
    .init(&shared_pool)
    .await
    .context("Cluster initialization failed")?;

  let plugin_manager = PluginManager::new(PathBuf::from("./plugins"));
  plugin_manager.load_plugins().await?;

  match config.mode {
    NodeMode::Main => {
      info!("Starting doseid in MAIN mode");

      // Register self as main node.
      // Use host.docker.internal so the Caddy container can reach doseid.
      let self_ip = "host.docker.internal".to_string();
      Node::register(self_ip, config.port as i16, true, &shared_pool).await?;

      // Start Caddy container, sync config, and watch for crashes
      caddy::ensure_running().await?;
      caddy::sync_config(&shared_pool).await;
      caddy::start_watcher(Arc::clone(&shared_pool));

      // Start stale node cleanup (removes workers with no heartbeat in 90s)
      let caddy_pool = Arc::clone(&shared_pool);
      start_stale_node_cleanup(Arc::clone(&shared_pool), move |_pool| {
        trigger_sync(Arc::clone(&caddy_pool));
      });
    }
    NodeMode::Worker => {
      let main_url = config
        .main_url
        .as_ref()
        .expect("DOSEID_MAIN_URL is required in worker mode");
      info!("Starting doseid in WORKER mode (main: {})", main_url);

      // Register with main node
      let client = reqwest::Client::new();
      let local_ip = get_local_ip().unwrap_or_else(|| "127.0.0.1".to_string());
      let resp = client
        .post(format!("{}/internal/nodes/register", main_url))
        .json(&serde_json::json!({
          "ip": local_ip,
          "port": config.port
        }))
        .send()
        .await
        .context("Failed to register with main node")?;

      if !resp.status().is_success() {
        anyhow::bail!("Failed to register with main node: {}", resp.status());
      }

      let registered_node: Node = resp.json().await.context("Failed to parse registration response")?;
      info!("Registered as node {}", registered_node.id);

      // Start heartbeat task
      start_heartbeat_task(main_url.clone(), registered_node.id);
    }
  }

  Job::start_server().await?;
  Container::start_event_listener().await?;
  Container::start_monitoring_server().await?;
  Http::start_server(config, &shared_pool).await?;
  Ok(())
}

/// Try to determine the local IP address of this machine.
fn get_local_ip() -> Option<String> {
  let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
  socket.connect("8.8.8.8:80").ok()?;
  socket.local_addr().ok().map(|addr| addr.ip().to_string())
}
