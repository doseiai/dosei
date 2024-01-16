use crate::server::token::schema::Token;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

pub async fn api_get_tokens(
  pool: Extension<Arc<Pool<Postgres>>>,
) -> Result<Json<Vec<Token>>, StatusCode> {
  match sqlx::query_as!(
    Token,
    "SELECT * FROM token WHERE owner_id = $1::uuid",
    Uuid::new_v4()
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
  Json(body): Json<TokenBody>,
) -> Result<Json<Token>, StatusCode> {
  let token = Token::new(body.name, body.days_until_expiration, Uuid::new_v4())
    .map_err(|_| StatusCode::BAD_REQUEST)?;
  Ok(Json(token))
}

#[derive(Deserialize)]
pub struct TokenBody {
  name: String,
  days_until_expiration: i32,
}
