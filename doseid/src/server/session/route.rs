use crate::config::Config;
use crate::server::integration::github::AccessTokenError;
use crate::server::session::schema::{Session, SessionCredentials};
use crate::server::session::validate_session;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use chrono::Utc;
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
  match sqlx::query!(
    "SELECT * FROM \"user\" WHERE (github ->> 'id')::bigint = $1",
    user.id
  )
  .fetch_one(&**pool)
  .await
  {
    Ok(rec) => {
      sqlx::query!(
        "UPDATE \"user\" SET github = $1, updated_at = $2  WHERE (github ->> 'id')::bigint = $3 RETURNING *",
        json!(user),
        Utc::now(),
        user.id,
      )
        .fetch_one(&**pool)
        .await
        .unwrap();
      info!("{:?}", rec);
    }
    Err(err) => {
      sqlx::query!(
        "
        INSERT INTO \"user\" (id, username, email, github, updated_at, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        ",
        Uuid::new_v4(),
        user.login,
        user.email,
        json!(user),
        Utc::now(),
        Utc::now()
      )
      .fetch_one(&**pool)
      .await
      .unwrap();
      error!("{}", err);
    }
  }
  let owner_id = Uuid::new_v4();
  let credentials =
    Session::new(&config, owner_id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok(Json(credentials.session_credentials()))
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
