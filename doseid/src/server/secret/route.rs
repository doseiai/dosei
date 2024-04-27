use crate::config::Config;
use crate::crypto::encrypt_value;
use crate::crypto::schema::SigningKey;
use crate::server::env;
use crate::server::session::validate_session;
use axum::http::StatusCode;
use axum::{Extension, Json};
use base64::engine::general_purpose;
use base64::Engine;
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

  let mut signing_key = SigningKey::new().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

  let nonce_base64 = general_purpose::STANDARD.encode(*signing_key.nonce.as_ref());
  let encrypted_value: Vec<u8> = encrypt_value(
    session.user_id,
    &body.value,
    &mut signing_key.key,
    signing_key.nonce,
  )
  .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

  let env_secret = env::schema::Env {
    id: Uuid::new_v4(),
    name: body.name.to_string(),
    value: general_purpose::STANDARD.encode(encrypted_value),
    key: Some(general_purpose::STANDARD.encode(signing_key.bytes)),
    nonce: Some(nonce_base64),
    service_id: None,
    deployment_id: None,
    owner_id: session.user_id,
    updated_at: Utc::now(),
    created_at: Utc::now(),
  };
  env_secret
    .save_secret(Arc::clone(&pool))
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

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

  let secret = env::schema::Env::get_secret(
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
