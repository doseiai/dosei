pub(crate) mod route;
mod schema;

use crate::config::Config;
use crate::server::session::schema::Session;
use crate::server::token::schema::Token;
use axum::http::{header, StatusCode};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use sqlx::{Pool, Postgres};
use std::collections::HashSet;
use std::sync::Arc;

const BEARER: &str = "Bearer ";

pub async fn validate_session(
  pool: Arc<Pool<Postgres>>,
  config: &'static Config,
  headers: axum::http::HeaderMap,
) -> Result<Session, StatusCode> {
  let authorization_header = headers
    .get(header::AUTHORIZATION)
    .ok_or(StatusCode::UNAUTHORIZED)?;
  let authorization = authorization_header
    .to_str()
    .map_err(|_| StatusCode::UNAUTHORIZED)?;
  if !authorization.starts_with(BEARER) {
    return Err(StatusCode::UNAUTHORIZED);
  }
  let bearer_token = authorization.trim_start_matches(BEARER);
  if bearer_token.starts_with("eyJhbGciOiJ") {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims = HashSet::with_capacity(0);
    validation.validate_exp = false;
    let token_message = jsonwebtoken::decode::<Session>(
      bearer_token,
      &DecodingKey::from_secret(config.jwt_secret.as_ref()),
      &validation,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;
    return Ok(token_message.claims);
  }
  let token = sqlx::query_as!(
    Token,
    "SELECT * FROM token WHERE value = $1::text and expires_at >= CURRENT_TIMESTAMP",
    bearer_token
  )
  .fetch_one(&*pool)
  .await
  .map_err(|_| StatusCode::UNAUTHORIZED)?;
  Ok(Session {
    owner_id: token.owner_id,
  })
}
