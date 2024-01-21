mod schema;

use crate::config::Config;
use crate::server::integration::github::CreateRepoError;
use crate::server::project::schema::{GitSource, Project};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::sync::Arc;
use tempfile::tempdir;
use tracing::{error, info};
use uuid::Uuid;

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
  // TODO: Find on db
  let access_token = &env::var("GITHUB_TEST_ACCESS_TOKEN").unwrap();
  let github_repo_response = github_integration
    .new_individual_repository(&body.name, None, access_token)
    .await
    .map_err(|e| match e {
      CreateRepoError::RequestError(_) => {
        // TODO: report to sentry or something
        StatusCode::INTERNAL_SERVER_ERROR
      }
      CreateRepoError::RepoExists => StatusCode::UNPROCESSABLE_ENTITY,
    })?;

  let temp_dir = tempdir().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
  let temp_path = temp_dir.path();
  let template_path = match body.path {
    None => temp_path.to_path_buf(),
    Some(path) => {
      let path_str = format!("{}/{}", temp_path.display(), path);
      Path::new(&path_str).to_path_buf()
    }
  };

  github_integration
    .github_clone(
      body.source_full_name,
      temp_path,
      body.branch.as_deref(),
      Some(access_token),
      None,
    )
    .await
    .map_err(|_| {
      error!("Github Clone Failed");
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let project = Project {
    id: Uuid::new_v4(),
    name: body.name.clone(),
    owner_id: body.owner_id,
    git_source: GitSource::Github,
    git_source_metadata: github_repo_response,
    updated_at: Default::default(),
    created_at: Default::default(),
  };
  match sqlx::query_as!(
      Project,
      r#"INSERT INTO project (id, name, owner_id, git_source, git_source_metadata, updated_at, created_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7)
       RETURNING id, name, owner_id, git_source AS "git_source!: GitSource", git_source_metadata, updated_at, created_at"#,
      project.id,
      project.name,
      project.owner_id,
      project.git_source as GitSource,
      project.git_source_metadata,
      project.updated_at,
      project.created_at,
    ).fetch_one(&**pool).await {
    Ok(recs) => {
      info!("{:?}", recs);
    },
    Err(err) => {
      error!("Error in creating project: {:?}", err);
      return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
  }

  github_integration
    .git_push(
      format!("Alw3ys/{}", body.name),
      &template_path,
      Some(access_token),
      None,
      "Alw3ys",
      "am@dosei.ai",
    )
    .await
    .map_err(|err| {
      error!("Error in creating project: {:?}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;
  drop(temp_dir);
  // TODO: Save secrets / envs
  // TODO: Assign domain
  Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct NewProjectFromClone {
  source_full_name: String,
  branch: Option<String>,
  path: Option<String>,
  private: Option<bool>,
  owner_id: Uuid,
  name: String,
  envs: Option<HashMap<String, String>>,
}