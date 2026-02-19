use crate::deployment::Deployment;
use crate::ingress::Ingress;
use crate::node::Node;
use bollard::container::{
  CreateContainerOptions, ListContainersOptions, RemoveContainerOptions, StartContainerOptions,
  StopContainerOptions,
};
use bollard::image::CreateImageOptions;
use bollard::models::HostConfig;
use bollard::Docker;
use futures_util::StreamExt;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

const CADDY_IMAGE: &str = "caddy:2";
pub const CADDY_CONTAINER_NAME: &str = "doseid-caddy";
const CADDY_ADMIN_URL: &str = "http://127.0.0.1:2019/load";

/// Ensure the Caddy container is running. Pulls the image if needed,
/// creates and starts the container with ports 80, 443, and 2019 exposed.
pub async fn ensure_running() -> anyhow::Result<()> {
  let docker = Docker::connect_with_socket_defaults()?;

  // Check if container already exists and is running
  let filters = HashMap::from([("name".to_string(), vec![CADDY_CONTAINER_NAME.to_string()])]);
  let containers = docker
    .list_containers(Some(ListContainersOptions {
      all: true,
      filters,
      ..Default::default()
    }))
    .await?;

  if let Some(container) = containers.first() {
    let state = container.state.as_deref().unwrap_or("");
    if state == "running" {
      info!("Caddy container already running");
      return Ok(());
    }
    // Container exists but not running — remove and recreate
    let id = container.id.as_deref().unwrap_or(CADDY_CONTAINER_NAME);
    warn!("Caddy container exists but state='{}', recreating", state);
    let _ = docker
      .stop_container(id, Some(StopContainerOptions { t: 5 }))
      .await;
    docker
      .remove_container(id, Some(RemoveContainerOptions { force: true, ..Default::default() }))
      .await?;
  }

  // Pull image
  info!("Pulling Caddy image: {}", CADDY_IMAGE);
  let mut pull_stream = docker.create_image(
    Some(CreateImageOptions {
      from_image: CADDY_IMAGE,
      ..Default::default()
    }),
    None,
    None,
  );
  while let Some(result) = pull_stream.next().await {
    match result {
      Ok(info) => {
        if let Some(status) = info.status {
          info!("Pull: {}", status);
        }
      }
      Err(e) => {
        error!("Pull error: {}", e);
        anyhow::bail!("Failed to pull Caddy image: {}", e);
      }
    }
  }

  // Use host network so Caddy binds directly to host ports (80, 443, 2019)
  // and doseid can reach the admin API at 127.0.0.1:2019
  let host_config = HostConfig {
    network_mode: Some("host".to_string()),
    restart_policy: Some(bollard::models::RestartPolicy {
      name: Some(bollard::models::RestartPolicyNameEnum::UNLESS_STOPPED),
      maximum_retry_count: None,
    }),
    ..Default::default()
  };

  let config = bollard::container::Config {
    image: Some(CADDY_IMAGE),
    host_config: Some(host_config),
    cmd: Some(vec!["caddy", "run", "--resume"]),
    ..Default::default()
  };

  let options = CreateContainerOptions {
    name: CADDY_CONTAINER_NAME,
    platform: None,
  };

  let container = docker.create_container(Some(options), config).await?;
  docker
    .start_container(&container.id, None::<StartContainerOptions<String>>)
    .await?;

  info!("Caddy container started: {}", container.id);
  Ok(())
}

/// Generate the full Caddy JSON config from the current DB state.
/// - App ingresses route directly to the container's host_port
/// - Ingresses without a deployment (e.g. cluster API domain) route to the main doseid node
pub async fn generate_config(pg_pool: &Pool<Postgres>) -> anyhow::Result<serde_json::Value> {
  let nodes = Node::list_active(pg_pool).await?;
  let ingresses = sqlx::query_as!(Ingress, "SELECT * FROM ingress")
    .fetch_all(pg_pool)
    .await?;

  let mut routes = Vec::new();

  for ingress in &ingresses {
    // Look up the latest deployment for this ingress's service
    let deployment = Deployment::get_by_service_id(ingress.service_id, pg_pool)
      .await
      .unwrap_or_default()
      .into_iter()
      .max_by_key(|d| d.created_at);

    let upstreams: Vec<serde_json::Value> = match deployment.and_then(|d| d.host_port) {
      // App ingress: route directly to container port on main node
      // (containers bind to 127.0.0.1, only reachable locally)
      Some(host_port) => {
        nodes
          .iter()
          .filter(|n| n.is_main)
          .map(|n| json!({ "dial": format!("{}:{}", n.ip, host_port) }))
          .collect()
      }
      // No deployment or no port: route to doseid (main node only)
      None => {
        nodes
          .iter()
          .filter(|n| n.is_main)
          .map(|n| json!({ "dial": format!("{}:{}", n.ip, n.port) }))
          .collect()
      }
    };

    if upstreams.is_empty() {
      continue;
    }

    routes.push(json!({
      "match": [{ "host": [&ingress.host] }],
      "handle": [{
        "handler": "reverse_proxy",
        "upstreams": upstreams
      }]
    }));
  }

  let config = json!({
    "apps": {
      "http": {
        "servers": {
          "srv0": {
            "listen": [":443"],
            "routes": routes
          }
        }
      }
    }
  });

  Ok(config)
}

/// Push the given Caddy JSON config to the Caddy admin API.
pub async fn push_config(config: &serde_json::Value) -> anyhow::Result<()> {
  let client = reqwest::Client::new();
  let resp = client
    .post(CADDY_ADMIN_URL)
    .json(config)
    .send()
    .await?;

  if resp.status().is_success() {
    info!("Caddy config updated successfully");
  } else {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    anyhow::bail!("Caddy config push failed ({}): {}", status, body);
  }
  Ok(())
}

/// Generate config from DB and push to Caddy. Logs errors but does not propagate.
/// Retries a few times to handle Caddy still starting up.
pub async fn sync_config(pg_pool: &Pool<Postgres>) {
  let config = match generate_config(pg_pool).await {
    Ok(config) => config,
    Err(e) => {
      error!("Failed to generate Caddy config: {}", e);
      return;
    }
  };

  for attempt in 1..=5 {
    match push_config(&config).await {
      Ok(_) => return,
      Err(e) => {
        if attempt < 5 {
          warn!("Caddy config push attempt {} failed: {}, retrying...", attempt, e);
          tokio::time::sleep(std::time::Duration::from_secs(attempt)).await;
        } else {
          error!("Failed to push Caddy config after {} attempts: {}", attempt, e);
        }
      }
    }
  }
}

/// Trigger a Caddy config sync from a spawned task (non-blocking).
pub fn trigger_sync(pg_pool: Arc<Pool<Postgres>>) {
  tokio::spawn(async move {
    sync_config(&pg_pool).await;
  });
}

/// Watch Docker events for the Caddy container dying and auto-restart it.
/// Re-pushes the config after restart so Caddy picks up where it left off.
pub fn start_watcher(pg_pool: Arc<Pool<Postgres>>) {
  use bollard::models::EventMessageTypeEnum;
  use bollard::system::EventsOptions;

  tokio::spawn(async move {
    let docker = Docker::connect_with_socket_defaults().unwrap();
    let filters = HashMap::from([
      ("type", vec!["container"]),
      ("event", vec!["die"]),
      ("container", vec![CADDY_CONTAINER_NAME]),
    ]);
    let mut stream = docker.events(Some(EventsOptions {
      filters,
      ..Default::default()
    }));

    while let Some(event_result) = stream.next().await {
      match event_result {
        Ok(event) => {
          if event.typ == Some(EventMessageTypeEnum::CONTAINER) {
            warn!("Caddy container died, restarting...");
            if let Err(e) = ensure_running().await {
              error!("Failed to restart Caddy: {}", e);
              continue;
            }
            // Re-push config after restart
            sync_config(&pg_pool).await;
          }
        }
        Err(e) => {
          error!("Caddy event watcher error: {}", e);
        }
      }
    }
  });
}
