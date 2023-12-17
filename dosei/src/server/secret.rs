use crate::schema::Secret;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use log::error;
use serde::Deserialize;
use sqlx::{Pool, Postgres, QueryBuilder};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

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
      error!("{:?}", err);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

pub async fn api_set_envs(
  pool: Extension<Arc<Pool<Postgres>>>,
  Path(params): Path<EnvsPathParams>,
  Json(body): Json<HashMap<String, String>>,
) -> Json<Vec<Secret>> {
  let mut secrets: Vec<Secret> = vec![];

  for (name, value) in body.into_iter() {
    // check db for this name and see if it exists
    let recs = sqlx::query_as!(
      Secret,
      r#"SELECT * FROM envs WHERE owner_id = $1::uuid and name = $2::text and value = $3::text"#,
      params.owner_id,
      name,
      value
    )
    .fetch_all(&**pool)
    .await
    .unwrap();

    // if nothing, set for update
    if recs.is_empty() {
      secrets.push(Secret {
        id: Uuid::new_v4(),
        name,
        value,
        owner_id: params.owner_id,
        project_id: params.project_id,
        updated_at: Default::default(),
        created_at: Default::default(),
      });
    }
  }

  let mut query_builder = QueryBuilder::new(
    "INSERT INTO envs (id, name, value, owner_id, project_id, updated_at, created_at) ",
  );
  query_builder.push_values(secrets, |mut qb, scr| {
    qb.push_bind(scr.id)
      .push_bind(scr.name)
      .push_bind(scr.value)
      .push_bind(scr.owner_id)
      .push_bind(scr.project_id)
      .push_bind(scr.updated_at)
      .push_bind(scr.created_at);
  });

  query_builder.build().execute(&**pool).await.unwrap();

  // todo
  // ideally should get just the list of env vars updated via the call
  match params.project_id {
    Some(_) => {
      let recs = sqlx::query_as!(
        Secret,
        r#"SELECT * FROM envs WHERE project_id = $1::uuid and owner_id = $2::uuid"#,
        params.project_id,
        params.owner_id
      )
      .fetch_all(&**pool)
      .await
      .unwrap();
      Json(recs)
    }
    None => {
      let recs = sqlx::query_as!(
        Secret,
        r#"SELECT * FROM envs WHERE owner_id = $1::uuid"#,
        params.owner_id
      )
      .fetch_all(&**pool)
      .await
      .unwrap();
      Json(recs)
    }
  }
}

#[derive(Deserialize, Debug)]
pub struct EnvsPathParams {
  owner_id: Uuid,
  project_id: Option<Uuid>,
}
