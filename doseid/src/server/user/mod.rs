use crate::server::user::schema::User;
use sqlx::{Pool, Postgres};
use std::error::Error;
use std::sync::Arc;
use uuid::Uuid;

pub(crate) mod route;
pub(crate) mod schema;

pub async fn get_user(id: Uuid, pool: Arc<Pool<Postgres>>) -> Result<User, Box<dyn Error>> {
  let user = sqlx::query_as!(User, "SELECT * FROM \"user\" WHERE id = $1::uuid", id)
    .fetch_one(&*pool)
    .await?;
  Ok(user)
}
