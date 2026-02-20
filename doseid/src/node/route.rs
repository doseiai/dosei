use crate::caddy;
use crate::node::Node;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RegisterRequest {
  pub ip: String,
  pub port: Option<i16>,
}

#[derive(Deserialize)]
pub struct HeartbeatRequest {
  pub node_id: Uuid,
}

pub async fn register(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<Node>), StatusCode> {
  let port = body.port.unwrap_or(8080);
  let node = Node::register(body.ip, port, false, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  // Update Caddy config with the new node
  caddy::trigger_sync(Arc::clone(&pg_pool));
  Ok((StatusCode::CREATED, Json(node)))
}

pub async fn heartbeat(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Json(body): Json<HeartbeatRequest>,
) -> Result<StatusCode, StatusCode> {
  Node::heartbeat(body.node_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;
  Ok(StatusCode::OK)
}

pub async fn list_nodes(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
) -> Result<(StatusCode, Json<Vec<Node>>), StatusCode> {
  let nodes = Node::list_active(&pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok((StatusCode::OK, Json(nodes)))
}

pub async fn delete_node(
  pg_pool: Extension<Arc<Pool<Postgres>>>,
  Path(node_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
  Node::delete(node_id, &pg_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  // Update Caddy config without the removed node
  caddy::trigger_sync(Arc::clone(&pg_pool));
  Ok(StatusCode::NO_CONTENT)
}
