use crate::config::Config;
use crate::container;
use crate::container::{build_docker_image, container_working_dir, export_dot_dosei};
use crate::server::deployment::schema;
use crate::server::deployment::schema::DeploymentStatus;
use crate::server::service::get_or_create_service;
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
  let session = validate_session(&config, headers).await?;

  let mut combined_data = Vec::new();
  while let Some(field) = multipart.next_field().await.unwrap() {
    let data = field.bytes().await.unwrap();
    combined_data.extend(data.clone());
  }

  let image_tag = format!("{}/{}", Uuid::new_v4(), Uuid::new_v4());
  build_docker_image(&image_tag, &combined_data)
    .await
    .map_err(|e| {
      error!("{}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let container_id = container::dosei_init(&image_tag).await.map_err(|e| {
    error!("{}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  let working_directory = container_working_dir(&container_id)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

  let dest_dir = tempdir().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  export_dot_dosei(&container_id, &working_directory, dest_dir.path())
    .await
    .map_err(|e| {
      error!("{}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let app = schema::App::import_from_dot_dosei(dest_dir.path()).map_err(|e| {
    error!("{}", e);
    StatusCode::INTERNAL_SERVER_ERROR
  })?;

  if let Some(name) = &app.name {
    let service = get_or_create_service(name, session.user_id, Arc::clone(&pool))
      .await
      .map_err(|e| {
        error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
      })?;
    let deployment = schema::Deployment::new(service.id, service.owner_id, &app, Arc::clone(&pool))
      .await
      .map_err(|e| {
        error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
      })?;
    container::run_deployment(&deployment, &image_tag)
      .await
      .map_err(|e| {
        error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
      })?;
    schema::Deployment::update_status(
      deployment.id,
      deployment.owner_id,
      DeploymentStatus::Ready,
      Arc::clone(&pool),
    )
    .await
    .map_err(|e| {
      error!("{}", e);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  }
  Ok(StatusCode::CREATED.into_response())
}
