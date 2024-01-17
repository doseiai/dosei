use crate::server::token::schema::Token;
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
) -> Result<Json<Token>, Response> {
  let token = Token::new(body.name, body.days_until_expiration, Uuid::new_v4()).map_err(|e| {
    (
      StatusCode::BAD_REQUEST,
      Json(json!({"message": e.to_string()})),
    )
      .into_response()
  })?;
  Ok(Json(token))
}

#[derive(Deserialize)]
pub struct TokenBody {
  name: String,
  days_until_expiration: i32,
}

pub async fn api_delete_token(
  pool: Extension<Arc<Pool<Postgres>>>,
  Path(token_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
  match sqlx::query!(
    "DELETE FROM token WHERE id = $1::uuid and owner_id = $2::uuid",
    token_id,
    token_id
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
