use crate::config::Config;
use crate::server::integration::github::AccessTokenError;
use crate::server::session::schema::{Session, SessionCredentials};
use crate::server::session::validate_session;
use crate::server::user::schema::User;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tracing::error;
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
  let user = match sqlx::query_as!(
    User,
    "SELECT * FROM \"user\" WHERE (github ->> 'id')::bigint = $1",
    user.id
  )
  .fetch_one(&**pool)
  .await
  {
    Ok(rec) => {
      sqlx::query_as!(
        User,
        "UPDATE \"user\" SET github = $1, updated_at = $2  WHERE (github ->> 'id')::bigint = $3 RETURNING *",
        json!(user),
        Utc::now(),
        user.id,
      )
        .fetch_one(&**pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    }
    Err(err) => {
      sqlx::query_as!(
        User,
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
      .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    }
  };
  let credentials =
    Session::new(&config, user.id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  sqlx::query!(
    "
    INSERT INTO session (id, token, refresh_token, owner_id, updated_at, created_at)
    VALUES ($1, $2, $3, $4, $5, $6)
    RETURNING *
    ",
    credentials.id,
    credentials.token,
    credentials.refresh_token,
    credentials.owner_id,
    credentials.updated_at,
    credentials.created_at,
  )
  .fetch_one(&**pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok(Json(credentials.session_credentials()))
}

pub async fn api_logout(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  Query(query): Query<LogoutQuery>,
) -> Result<Response, StatusCode> {
  let session = validate_session(Arc::clone(&pool), &config, headers).await?;

  if sqlx::query_as!(
    Session,
    "SELECT * FROM session WHERE id = $1::uuid and owner_id = $2::uuid",
    query.session_id,
    session.owner_id
  )
  .fetch_one(&**pool)
  .await
  .is_err()
  {
    return Ok(
      (
        StatusCode::NOT_FOUND,
        Json(json!({"message": "Session not found."})),
      )
        .into_response(),
    );
  }
  if let Some(true) = query.revoke_all_sessions {
    sqlx::query!(
      "DELETE FROM session WHERE owner_id = $1::uuid",
      session.owner_id
    )
    .execute(&**pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    return Ok(
      (
        StatusCode::OK,
        Json(json!({"message": "You have successfully logout from all your active sessions."})),
      )
        .into_response(),
    );
  }
  sqlx::query!(
    "DELETE FROM session WHERE id = $1::uuid and owner_id = $2::uuid",
    query.session_id,
    session.owner_id
  )
  .execute(&**pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
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
