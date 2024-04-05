use crate::config::Config;
use crate::server::deployment::schema::Deployment;
use crate::server::deployment::schema::DeploymentStatus;
use crate::server::project::schema::Project;
use crate::server::project::GitSource;
use crate::server::session::validate_session;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use sqlx::{Pool, Postgres};
use std::sync::Arc;

pub async fn api_list_projects(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
) -> Result<Json<Vec<Project>>, StatusCode> {
  let session = validate_session(Arc::clone(&pool), &config, headers).await?;
  match sqlx::query_as!(
    Project,
    r#"SELECT id, name, owner_id, git_source AS "git_source!: GitSource", git_source_metadata, updated_at, created_at FROM project WHERE owner_id = $1::uuid"#,
    session.owner_id
  )
  .fetch_all(&**pool)
  .await
  {
    Ok(projects) => Ok(Json(projects)),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}

pub async fn api_list_project_deployments(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  Path((owner_name, project_name)): Path<(String, String)>,
  headers: axum::http::HeaderMap,
) -> Result<Json<Vec<Deployment>>, StatusCode> {
  let session = validate_session(Arc::clone(&pool), &config, headers).await?;
  match sqlx::query_as!(
    Deployment,
    r#"
    SELECT d.id, d.commit_id, d.commit_metadata, d.project_id, d.owner_id, d.status AS "status!: DeploymentStatus", d.build_logs, d.exposed_port, d.internal_port, d.updated_at, d.created_at
    FROM deployment d
    INNER JOIN project p ON p.id = d.project_id
    WHERE p.name = $1 AND d.owner_id = $2::uuid
    "#,
    project_name,
    session.owner_id,
  )
  .fetch_all(&**pool)
  .await
  {
    Ok(deployments) => Ok(Json(deployments)),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}
