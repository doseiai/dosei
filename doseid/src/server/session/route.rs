use crate::config::Config;
use crate::server::session::schema::{Session, SessionCredentials};
use crate::server::session::validate_session;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

pub async fn login_username_password(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  Json(body): Json<UsernamePasswordLoginBody>,
) -> Result<Json<SessionCredentials>, StatusCode> {
  let record = sqlx::query!(
    "
    SELECT account.id, account.name, \"user\".password
    FROM account
    JOIN \"user\" ON account.id = \"user\".id
    WHERE account.name = $1 AND account.type = 'individual'
    ",
    body.username
  )
  .fetch_one(&**pool)
  .await
  .map_err(|_| StatusCode::NOT_FOUND)?;

  if record.password.is_none() {
    return Err(StatusCode::UNAUTHORIZED);
  }
  if !bcrypt::verify(&body.password, &record.password.unwrap())
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
  {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let credentials =
    Session::new(&config, record.id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  sqlx::query!(
    "
    INSERT INTO session (id, token, refresh_token, user_id, updated_at, created_at)
    VALUES ($1, $2, $3, $4, $5, $6)
    RETURNING *
    ",
    credentials.id,
    credentials.token,
    credentials.refresh_token,
    credentials.user_id,
    credentials.updated_at,
    credentials.created_at,
  )
  .fetch_one(&**pool)
  .await
  .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  Ok(Json(credentials.session_credentials()))
}

#[derive(Deserialize)]
pub struct UsernamePasswordLoginBody {
  username: String,
  password: String,
}

pub async fn logout(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  Query(query): Query<LogoutQuery>,
) -> Result<Response, StatusCode> {
  let session = validate_session(&config, headers).await?;

  if sqlx::query_as!(
    Session,
    "SELECT * FROM session WHERE id = $1::uuid and user_id = $2::uuid",
    query.session_id,
    session.user_id
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
      "DELETE FROM session WHERE user_id = $1::uuid",
      session.user_id
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
    "DELETE FROM session WHERE id = $1::uuid and user_id = $2::uuid",
    query.session_id,
    session.user_id
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
pub struct LogoutQuery {
  session_id: Uuid,
  revoke_all_sessions: Option<bool>,
}
