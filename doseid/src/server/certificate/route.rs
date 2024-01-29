use crate::config::Config;
use crate::server::certificate::schema::Certificate;
use crate::server::certificate::{
  create_acme_account, create_acme_certificate, get_certificate_order,
};
use crate::server::session::validate_session;
use crate::server::user::get_user;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use chrono::Utc;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

pub async fn api_new_certificate(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  Json(body): Json<CertificateBody>,
) -> Result<Json<Certificate>, Response> {
  let session = validate_session(Arc::clone(&pool), &config, headers)
    .await
    .map_err(|e| e.into_response())?;
  let user = get_user(session.owner_id, Arc::clone(&pool))
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
  info!("{}", user.email);
  let acme_account_credentials = create_acme_account(&user.email)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
  let token = create_acme_certificate(&body.domain_name, acme_account_credentials)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())?;
  info!("{}", token);
  let certificate = Certificate {
    id: Uuid::new_v4(),
    domain_name: body.domain_name,
    certificate: "".to_string(),
    private_key: "".to_string(),
    expires_at: Utc::now(),
    owner_id: Default::default(),
    updated_at: Utc::now(),
    created_at: Utc::now(),
  };
  info!("{:?}", certificate);
  Ok(Json(certificate))
}

#[derive(Deserialize)]
pub struct CertificateBody {
  domain_name: String,
}

pub async fn api_http01_challenge(Path(token): Path<String>) -> Result<String, Response> {
  if let Some(token_value) = get_certificate_order(token).await {
    return Ok(token_value);
  }
  Err(StatusCode::NOT_FOUND.into_response())
}
