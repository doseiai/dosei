use crate::config::Config;
use crate::server::session::validate_session;
use crate::server::user::schema::User;
use axum::http::StatusCode;
use axum::{Extension, Json};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub async fn user(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
) -> Result<Json<User>, StatusCode> {
  let session = validate_session(&config, headers).await?;
  let record = sqlx::query_as!(
    User,
    "
    SELECT account.id, account.name, account.updated_at, account.created_at
    FROM account
    JOIN \"user\" ON account.id = \"user\".id
    WHERE account.id = $1 AND account.type = 'individual'
    ",
    session.user_id
  )
  .fetch_one(&**pool)
  .await
  .map_err(|_| StatusCode::NOT_FOUND)?;
  Ok(Json(record))
}
