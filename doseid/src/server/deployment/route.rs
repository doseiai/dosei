use crate::config::Config;
use crate::container::build_docker_image;
use crate::server::session::validate_session;
use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Extension;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tempfile::tempdir;
use tracing::error;
use uuid::Uuid;

pub async fn deploy(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  mut multipart: Multipart,
) -> Result<Response, StatusCode> {
  validate_session(&config, headers).await?;

  let mut combined_data = Vec::new();
  while let Some(field) = multipart.next_field().await.unwrap() {
    let data = field.bytes().await.unwrap();
    combined_data.extend(data.clone());
  }

  let image_tag = format!("{}/{}", Uuid::new_v4(), Uuid::new_v4());
  let build_logs = build_docker_image(&image_tag, &combined_data)
    .await
    .map_err(|e| {
      error!("{}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  println!("{:?}", build_logs);

  let temp_dir = tempdir().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  let temp_path = temp_dir.path();
  dosei_util::extract_tar_gz_from_memory(&combined_data, temp_path)
    .await
    .map_err(|e| {
      error!("{}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(StatusCode::CREATED.into_response())
}
