use crate::schema::Secret;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use log::error;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

pub async fn api_get_envs(
  pool: Extension<Arc<Pool<Postgres>>>,
  Path(params): Path<EnvsPathParams>,
) -> Result<Json<Vec<Secret>>, StatusCode> {
  let result = match params.project_id {
    Some(project_id) => {
      sqlx::query_as!(
        Secret,
        "SELECT * FROM envs WHERE project_id = $1::uuid and owner_id = $2::uuid",
        project_id,
        params.owner_id
      )
      .fetch_all(&**pool)
      .await
    }
    None => {
      sqlx::query_as!(
        Secret,
        "SELECT * FROM envs WHERE owner_id = $1::uuid",
        params.owner_id
      )
      .fetch_all(&**pool)
      .await
    }
  };
  match result {
    Ok(recs) => Ok(Json(recs)),
    Err(err) => {
      error!("Error in retrieving secret: {:?}", err);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

pub async fn api_set_envs(
  pool: Extension<Arc<Pool<Postgres>>>,
  Path(params): Path<EnvsPathParams>,
  Json(body): Json<HashMap<String, String>>,
) -> Result<Json<Vec<Secret>>, StatusCode> {
  let mut updated_secrets: Vec<Secret> = Vec::new();

  for (name, value) in body {
    let query = sqlx::query_as!(
      Secret,
      "INSERT INTO envs (id, name, value, owner_id, project_id, updated_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)
       ON CONFLICT (owner_id, project_id, name) DO UPDATE
       SET value = EXCLUDED.value, updated_at = EXCLUDED.updated_at
       RETURNING *",
      Uuid::new_v4(),
      name,
      value,
      params.owner_id,
      params.project_id,
      Utc::now(),
      Utc::now()
    );

    match query.fetch_one(&**pool).await {
      Ok(secret) => updated_secrets.push(secret),
      Err(err) => {
        error!("Error in upserting and retrieving secret: {:?}", err);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
      }
    }
  }
  Ok(Json(updated_secrets))
}

#[derive(Deserialize, Debug)]
pub struct EnvsPathParams {
  owner_id: Uuid,
  project_id: Option<Uuid>,
}
