use crate::config::Config;
use crate::deployment::_build_internal;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub async fn api_build(
  config: Extension<&'static Config>,
  Json(build): Json<Build>,
) -> Result<StatusCode, StatusCode> {
  _build_internal(
    &config,
    &format!("{}/{}", build.owner_id, build.project_id),
    &build.deployment_id,
  )
  .await;
  Ok(StatusCode::OK)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Build {
  owner_id: Uuid,
  project_id: Uuid,
  deployment_id: String,
}
