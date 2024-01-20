use crate::config::Config;
use axum::http::header;
use axum::http::StatusCode;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

const BEARER: &str = "Bearer ";

pub async fn validate_session(
  config: &'static Config,
  headers: axum::http::HeaderMap,
) -> Result<Session, StatusCode> {
  let authorization_header = match headers.get(header::AUTHORIZATION) {
    Some(v) => v,
    None => return Err(StatusCode::UNAUTHORIZED),
  };
  let authorization = match authorization_header.to_str() {
    Ok(v) => v,
    Err(_) => return Err(StatusCode::UNAUTHORIZED),
  };
  if !authorization.starts_with(BEARER) {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let token = authorization.trim_start_matches(BEARER);
  if token.starts_with("eyJhbGciOiJ") {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims = HashSet::with_capacity(0);
    validation.validate_exp = false;
    let token_message = jsonwebtoken::decode::<Session>(
      token,
      &DecodingKey::from_secret(config.jwt_secret.as_ref()),
      &validation,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;
    return Ok(token_message.claims);
  } else {
    // TODO: Look for access token on db
  }
  Err(StatusCode::FORBIDDEN)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
  owner_id: Uuid,
}
