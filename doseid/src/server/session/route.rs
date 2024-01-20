use crate::config::Config;
use crate::server::session::schema::SessionCredentials;
use crate::server::session::validate_session;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

pub async fn api_auth_github_cli(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
) -> Result<Json<SessionCredentials>, StatusCode> {
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
pub struct LogoutQuery {
  session_id: Uuid,
  revoke_all_sessions: Option<bool>,
}
