pub mod route;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Node {
  pub id: Uuid,
  pub ip: String,
  pub port: i16,
  pub is_main: bool,
  pub last_heartbeat_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

impl Node {
  pub async fn register(
    ip: String,
    port: i16,
    is_main: bool,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Self> {
    // Upsert: if a node with the same IP already exists, update it
    let node = sqlx::query_as!(
      Node,
      "INSERT INTO node (id, ip, port, is_main, last_heartbeat_at, created_at)
       VALUES ($1, $2, $3, $4, $5, $6)
       ON CONFLICT (ip) DO UPDATE SET
         port = EXCLUDED.port,
         is_main = EXCLUDED.is_main,
         last_heartbeat_at = EXCLUDED.last_heartbeat_at
       RETURNING *",
      Uuid::new_v4(),
      ip,
      port,
      is_main,
      Utc::now(),
      Utc::now(),
    )
    .fetch_one(pg_pool)
    .await?;
    info!("Registered node: {} ({}:{})", node.id, node.ip, node.port);
    Ok(node)
  }

  pub async fn heartbeat(node_id: Uuid, pg_pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let result = sqlx::query!(
      "UPDATE node SET last_heartbeat_at = $1 WHERE id = $2",
      Utc::now(),
      node_id,
    )
    .execute(pg_pool)
    .await?;
    if result.rows_affected() == 0 {
      anyhow::bail!("Node {} not found", node_id);
    }
    Ok(())
  }

  pub async fn list_active(pg_pool: &Pool<Postgres>) -> anyhow::Result<Vec<Self>> {
    Ok(
      sqlx::query_as!(Node, "SELECT * FROM node ORDER BY created_at ASC")
        .fetch_all(pg_pool)
        .await?,
    )
  }

  pub async fn delete(node_id: Uuid, pg_pool: &Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::query!("DELETE FROM node WHERE id = $1", node_id)
      .execute(pg_pool)
      .await?;
    info!("Deleted node: {}", node_id);
    Ok(())
  }

  /// Remove nodes that haven't sent a heartbeat in the last `stale_secs` seconds.
  /// Returns the number of removed nodes.
  pub async fn remove_stale(stale_secs: i64, pg_pool: &Pool<Postgres>) -> anyhow::Result<u64> {
    let cutoff = Utc::now() - chrono::Duration::seconds(stale_secs);
    let result = sqlx::query!(
      "DELETE FROM node WHERE last_heartbeat_at < $1 AND is_main = false",
      cutoff,
    )
    .execute(pg_pool)
    .await?;
    let removed = result.rows_affected();
    if removed > 0 {
      warn!("Removed {} stale node(s)", removed);
    }
    Ok(removed)
  }
}

/// Start the stale node cleanup task (runs on main node).
/// Checks every 60s, removes nodes with no heartbeat in 90s.
/// Calls `on_change` whenever stale nodes are removed.
pub fn start_stale_node_cleanup<F>(pg_pool: Arc<Pool<Postgres>>, on_change: F)
where
  F: Fn(Arc<Pool<Postgres>>) + Send + Sync + 'static,
{
  let on_change = Arc::new(on_change);
  tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
      interval.tick().await;
      match Node::remove_stale(90, &pg_pool).await {
        Ok(removed) if removed > 0 => {
          info!("Stale cleanup removed {} node(s), updating Caddy config", removed);
          on_change(Arc::clone(&pg_pool));
        }
        Err(e) => error!("Stale node cleanup error: {}", e),
        _ => {}
      }
    }
  });
}

/// Start the heartbeat task (runs on worker node).
/// Sends heartbeat every 30s to the main node.
pub fn start_heartbeat_task(main_url: String, node_id: Uuid) {
  tokio::spawn(async move {
    let client = reqwest::Client::new();
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
      interval.tick().await;
      let url = format!("{}/internal/nodes/heartbeat", main_url);
      match client
        .post(&url)
        .json(&serde_json::json!({ "node_id": node_id }))
        .send()
        .await
      {
        Ok(resp) if resp.status().is_success() => {
          info!("Heartbeat sent successfully");
        }
        Ok(resp) => {
          warn!("Heartbeat response: {}", resp.status());
        }
        Err(e) => {
          error!("Heartbeat failed: {}", e);
        }
      }
    }
  });
}
