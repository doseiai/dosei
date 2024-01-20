use crate::config::Config;
use crate::server::token::schema::Token;
use crate::session::validate_session;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Deserialize;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

pub async fn api_get_tokens(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
) -> Result<Json<Vec<Token>>, StatusCode> {
  let session = validate_session(&config, headers).await?;
  match sqlx::query_as!(
    Token,
    "SELECT * FROM token WHERE owner_id = $1::uuid",
    session.owner_id
  )
  .fetch_all(&**pool)
  .await
  {
    Ok(recs) => Ok(Json(recs)),
    Err(err) => {
      error!("Error in retrieving tokens: {:?}", err);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

pub async fn api_set_token(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  Json(body): Json<TokenBody>,
) -> Result<Json<Token>, Response> {
  let session = validate_session(&config, headers)
    .await
    .map_err(|e| e.into_response())?;
  let token = Token::new(body.name, body.days_until_expiration, session.owner_id).map_err(|e| {
    (
      StatusCode::BAD_REQUEST,
      Json(json!({"message": e.to_string()})),
    )
      .into_response()
  })?;

  match sqlx::query_as!(
    Token,
    "
    INSERT INTO token (id, name, value, owner_id, expires_in, updated_at, created_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    RETURNING *
    ",
    token.id,
    token.name,
    token.value,
    token.owner_id,
    token.expires_in,
    token.updated_at,
    token.created_at
  )
  .fetch_one(&**pool)
  .await
  {
    Ok(recs) => Ok(Json(recs)),
    Err(err) => {
      error!("Error in creating token: {:?}", err);
      Err(StatusCode::INTERNAL_SERVER_ERROR.into_response())
    }
  }
}

#[derive(Deserialize)]
pub struct TokenBody {
  name: String,
  days_until_expiration: i32,
}

pub async fn api_delete_token(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  Path(token_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
  let session = validate_session(&config, headers).await?;
  match sqlx::query!(
    "DELETE FROM token WHERE id = $1::uuid and owner_id = $2::uuid",
    token_id,
    session.owner_id
  )
  .execute(&**pool)
  .await
  {
    Ok(res) => {
      if res.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
      } else {
        Ok(StatusCode::OK)
      }
    }
    Err(err) => {
      error!("Error in deleting token: {:?}", err);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}
