use crate::config::Config;
use crate::server::env;
use crate::server::session::validate_session;
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Secret {
  name: String,
  value: String,
}

pub async fn set_secret(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  Json(body): Json<Secret>,
) -> Result<Json<Secret>, StatusCode> {
  let session = validate_session(&config, headers).await?;

  // Make sure secret value is something like DOSEI_SECRET_A, not just DOSEI_SECRET_
  if !dosei_util::secret::is_secret_env(&body.name) {
    return Err(StatusCode::BAD_REQUEST);
  }

  let mut env_secret = env::schema::Env {
    id: Uuid::new_v4(),
    name: body.name.to_string(),
    value: body.value,
    key: None,
    nonce: None,
    service_id: None,
    deployment_id: None,
    owner_id: session.user_id,
    updated_at: Utc::now(),
    created_at: Utc::now(),
  };
  env_secret
    .save_secret_encrypted(Arc::clone(&pool))
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(Json(Secret {
    name: env_secret.name,
    value: env_secret.value,
  }))
}

pub async fn get_secret(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  Json(body): Json<Secret>,
) -> Result<Json<Secret>, StatusCode> {
  let session = validate_session(&config, headers).await?;

  let secret = env::schema::Env::get_secret_decrypted(
    body.name.clone(),
    body.value.clone(),
    session.user_id,
    Arc::clone(&pool),
  )
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  Ok(Json(Secret {
    name: secret.name,
    value: secret.value,
  }))
}
