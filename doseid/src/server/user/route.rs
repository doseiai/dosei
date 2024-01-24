use crate::config::Config;
use crate::server::session::validate_session;
use crate::server::user::get_user;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

pub async fn api_get_user(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
) -> Result<Json<User>, StatusCode> {
  let session = validate_session(Arc::clone(&pool), &config, headers).await?;
  let user = get_user(session.owner_id, Arc::clone(&pool))
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok(Json(User {
    id: user.id,
    username: user.username,
    email: user.email,
  }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
  id: Uuid,
  username: String,
  email: String,
}
