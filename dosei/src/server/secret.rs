use crate::schema::Secret;
use axum::extract::Query;
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::{Pool, Postgres, QueryBuilder};
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
  let mut secrets: Vec<Secret> = vec![];

  for (name, value) in body.into_iter() {
    secrets.push(Secret {
      id: Uuid::new_v4(),
      name,
      value,
      owner_id: query.owner_name,
      project_id: query.project_name,
      updated_at: Default::default(),
      created_at: Default::default(),
    });
  }

  let mut query_builder = QueryBuilder::new(
    "INSERT INTO envs (id, name, value, owner_id, project_id, updated_at, created_at) ",
  );
  query_builder.push_values(secrets, |mut b, new_secret| {
    b.push_bind(new_secret.id)
      .push_bind(new_secret.name)
      .push_bind(new_secret.value)
      .push_bind(new_secret.owner_id)
      .push_bind(new_secret.project_id)
      .push_bind(new_secret.updated_at)
      .push_bind(new_secret.created_at);
  });

  let query = query_builder.build();
  query.execute(&**pool).await.unwrap();

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
