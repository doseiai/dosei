mod app;

use crate::deployment::app::import_dosei_app;
use crate::docker::build_image;
use crate::git::github::GithubIntegration;
use std::path::Path;
use tempfile::tempdir;
use tracing::info;
use uuid::Uuid;

pub async fn build_from_github(
  github_integration: &'static GithubIntegration,
  deployment_id: String,
  repo_full_name: String,
  installation_id: i64,
) {
  let temp_dir = tempdir().unwrap();
  let temp_path = temp_dir.path();

  if let Err(err) = github_integration
    .github_clone(repo_full_name, temp_path, None, None, Some(installation_id))
    .await
  {
    eprintln!("ERROR: {}", err);
    err
      .chain()
      .skip(1)
      .for_each(|cause| eprintln!("because: {}", cause));
    return;
  }

  // build(Uuid::new_v4(), Uuid::new_v4(), deployment_id, temp_path).await;
  drop(temp_dir);
}

async fn build(owner_id: Uuid, project_id: Uuid, deployment_id: String, folder_path: &Path) {
  let detected_docker_file = dosei_util::package_manager::_resolve_docker(folder_path);
  if !detected_docker_file {
    todo!("Provision docker file template");
  }
  info!("Detected `Dockerfile`");
  let image_name = &format!("{}/{}", owner_id, project_id);
  let image_tag = &deployment_id;
  if let Ok(app) = import_dosei_app(image_name, image_tag, folder_path).await {
    todo!("Implement DoseiApp Deployment");
  }
  build_image(image_name, image_tag, folder_path).await;
}
