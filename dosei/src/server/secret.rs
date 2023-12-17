use crate::schema::Secret;
use axum::extract::Query;
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub async fn api_get_envs(pool: Extension<Arc<Pool<Postgres>>>) -> Json<Vec<Secret>> {
  let recs = sqlx::query_as!(Secret, "SELECT * from envs")
    .fetch_all(&**pool)
    .await
    .unwrap();
  Json(recs)
}

pub async fn api_set_envs(
  pool: Extension<Arc<Pool<Postgres>>>,
  Query(query): Query<SetEnvsQueryParams>,
  Json(body): Json<HashMap<String, String>>,
) -> Json<Vec<Secret>> {
  for (name, value) in body.into_iter() {
    let secret = Secret {
      id: Uuid::new_v4(),
      name,
      value,
      owner_id: query.owner_name,
      project_id: query.project_name,
      updated_at: Default::default(),
      created_at: Default::default(),
    };
    let _rec = sqlx::query_as!(
      Secret,
      r#"
      INSERT INTO envs (id, name, value, owner_id, project_id, updated_at, created_at)
      VALUES ($1, $2, $3, $4, $5, $6, $7)
      RETURNING *
      "#,
      secret.id,
      secret.name,
      secret.value,
      secret.owner_id,
      secret.project_id,
      secret.updated_at,
      secret.created_at
    )
    .fetch_one(&**pool)
    .await
    .unwrap();
  }
  let recs = sqlx::query_as!(Secret, "SELECT * from envs")
    .fetch_all(&**pool)
    .await
    .unwrap();
  Json(recs)
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct SetEnvsQueryParams {
  owner_name: Uuid,
  project_name: Uuid,
}
