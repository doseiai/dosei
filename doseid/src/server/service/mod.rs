use crate::server::service::schema::Service;
use chrono::Utc;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use uuid::Uuid;

pub(crate) mod schema;

pub async fn get_or_create_service(
  name: &str,
  owner_id: Uuid,
  pool: Arc<Pool<Postgres>>,
) -> anyhow::Result<Service> {
  let service = sqlx::query_as!(
    Service,
    "SELECT * FROM service WHERE name = $1 AND owner_id = $2::uuid",
    name,
    owner_id
  )
  .fetch_optional(&*pool)
  .await?;
  if let Some(service) = service {
    return Ok(service);
  }
  let service = sqlx::query_as!(
    Service,
    "
      INSERT INTO service (id, name, owner_id, updated_at, created_at)
      VALUES ($1::uuid, $2, $3::uuid, $4, $5)
      RETURNING *
      ",
    Uuid::new_v4(),
    name,
    owner_id,
    Utc::now(),
    Utc::now()
  )
  .fetch_one(&*pool)
  .await?;
  Ok(service)
}
