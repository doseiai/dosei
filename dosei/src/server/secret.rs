use crate::schema::Secret;
use axum::extract::Query;
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;

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
) -> Json<HashMap<String, String>> {
  println!("{:?}", body);
  println!("{query:?}");
  let rec = HashMap::new();
  Json(rec)
}

#[derive(Deserialize, Debug)]
pub struct SetEnvsQueryParams {
  owner_name: Option<String>,
  project_name: Option<String>,
}
