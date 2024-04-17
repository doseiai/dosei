pub(crate) mod route;
mod schema;

use crate::config::Config;
use crate::server::session::schema::SessionToken;
use axum::http::{header, StatusCode};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use std::collections::HashSet;

const BEARER: &str = "Bearer ";

pub async fn validate_session(
  config: &'static Config,
  headers: axum::http::HeaderMap,
) -> Result<SessionToken, StatusCode> {
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
  if jsonwebtoken::decode_header(bearer_token).is_ok() {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims = HashSet::with_capacity(0);
    validation.validate_exp = false;
    let token_message = jsonwebtoken::decode::<SessionToken>(
      bearer_token,
      &DecodingKey::from_secret(config.jwt_secret.as_ref()),
      &validation,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;
    return Ok(token_message.claims);
  }
  Err(StatusCode::UNAUTHORIZED)
}
