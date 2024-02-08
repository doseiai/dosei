use crate::config::Config;
use crate::server::session::validate_session;
use crate::server::token::route::TokenBody;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub async fn api_deploy(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  Json(body): Json<TokenBody>,
) -> Result<Json<()>, Response> {
  let session = validate_session(Arc::clone(&pool), &config, headers)
    .await
    .map_err(|e| e.into_response())?;
  Ok(Json({}))
}
