use crate::config::Config;
use crate::server::integration::github::AccessTokenError;
use crate::server::session::schema::SessionCredentials;
use crate::server::session::validate_session;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

pub async fn api_auth_github_cli(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  Query(query): Query<LoginQuery>,
) -> Result<Json<SessionCredentials>, StatusCode> {
  let github_integration = match config.github_integration.as_ref() {
    Some(github) => github,
    None => {
      error!("Github integration not enabled");
      return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
  };
  let access_token = github_integration
    .get_user_access_token(query.code)
    .await
    .map_err(|e| match e {
      AccessTokenError::BadVerificationCode => StatusCode::UNAUTHORIZED,
      _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;
  let user = github_integration
    .get_user(&access_token)
    .await
    .map_err(|e| {
      error!("{}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  let emails = github_integration
    .get_user_emails(&access_token)
    .await
    .map_err(|e| {
      error!("{}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  info!("{:?}", emails);
  let owner_id = Uuid::new_v4();
  let credentials =
    SessionCredentials::new(&config, owner_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok(Json(credentials))
}

pub async fn api_logout(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  Query(query): Query<LogoutQuery>,
) -> Result<Response, StatusCode> {
  let session = validate_session(Arc::clone(&pool), &config, headers).await?;
  // TODO: Find matching session otherwise return error 404
  // return Ok(
  //   (
  //     StatusCode::NOT_FOUND,
  //     Json(json!({"message": "Session not found."})),
  //   )
  //     .into_response(),
  // );
  if let Some(true) = query.revoke_all_sessions {
    // TODO: Call db and remove sessions
    return Ok(
      (
        StatusCode::OK,
        Json(json!({"message": "You have successfully logout from all your active sessions."})),
      )
        .into_response(),
    );
  }
  // TODO: Call db and remove session
  Ok(
    (
      StatusCode::OK,
      Json(json!({"message": "You have successfully logout."})),
    )
      .into_response(),
  )
}

#[derive(Deserialize)]
pub struct LoginQuery {
  code: String,
}

#[derive(Deserialize)]
pub struct LogoutQuery {
  session_id: Uuid,
  revoke_all_sessions: Option<bool>,
}
