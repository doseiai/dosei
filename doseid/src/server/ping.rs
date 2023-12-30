use axum::http::StatusCode;
use axum::Extension;
use bollard::Docker;
use sqlx::{Connection, Pool, Postgres};
use std::sync::Arc;

pub async fn api_ping(pool: Extension<Arc<Pool<Postgres>>>) -> Result<StatusCode, StatusCode> {
  let mut conn = pool
    .acquire()
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
  conn
    .ping()
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

  let docker =
    Docker::connect_with_socket_defaults().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
  docker
    .ping()
    .await
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
  Ok(StatusCode::OK)
}
