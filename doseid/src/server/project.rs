use crate::config::Config;
use crate::git::github::CreateRepoError;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;
use tracing::error;

pub async fn api_new_project(
  config: Extension<&'static Config>,
  pool: Extension<Arc<Pool<Postgres>>>,
  Json(body): Json<NewProjectFromClone>,
) -> Result<StatusCode, StatusCode> {
  let github_integration = match config.github_integration.as_ref() {
    Some(github) => github,
    None => {
      error!("Github integration not enabled");
      return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
  };
  let access_token = "TODO:REPLACEWITH_REAL_TOKEN";
  github_integration
    .new_individual_repo(&body.name, None, access_token)
    .await
    .map_err(|e| {
      return match e {
        CreateRepoError::RequestError(_) => {
          // TODO: report to sentry or something
          Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        CreateRepoError::RepoExists => Err(StatusCode::UNPROCESSABLE_ENTITY),
      };
    })?;

  let temp_dir = tempdir();
  let temp_path = temp_dir.path();
  let template_path = format!("{}/{}", temp_path, &body.path.unwrap());

  github_integration
    .github_clone(
      body.source_full_name,
      temp_path,
      Some(&body.branch.unwrap()),
      Some(access_token),
      None,
    )
    .await?;

  // TODO: Assign domain

  // TODO: Save secrets / envs

  // TODO: Git push
  Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct NewProjectFromClone {
  source_full_name: String,
  branch: Option<String>,
  path: Option<String>,
  #[serde(default = "default_private")]
  private: Option<bool>,
  owner: String,
  name: String,
  envs: Option<HashMap<String, String>>,
}
