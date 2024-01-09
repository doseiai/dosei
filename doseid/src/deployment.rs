mod app;

use crate::deployment::app::import_dosei_app;
use crate::docker::build_image;
use crate::git::github::GithubIntegration;
use std::path::Path;
use tempfile::tempdir;
use tracing::{error, info};
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
    error!("{}", err);
    return;
  }

  // build(Uuid::new_v4(), Uuid::new_v4(), deployment_id, temp_path).await;
  drop(temp_dir);
}

async fn build(owner_id: Uuid, project_id: Uuid, deployment_id: String, folder_path: &Path) {
  let detected_docker_file = dosei_util::package_manager::_resolve_docker(folder_path);
  if !detected_docker_file {
    // TODO: Implement docker file templates.
    error!("Failed to detect `Dockerfile`");
    return;
  }
  info!("Detected `Dockerfile`");
  let image_name = &format!("{}/{}", owner_id, project_id);
  let image_tag = &deployment_id;
  if let Ok(app) = import_dosei_app(image_name, image_tag, folder_path).await {
    // TODO: Implement DoseiApp Deployment
  }
  build_image(image_name, image_tag, folder_path).await;
}

mod tests {
  use crate::config::Config;
  use crate::deployment::build;
  use crate::git::git_clone;
  use git2::Repository;
  use home::home_dir;
  use once_cell::sync::Lazy;
  use std::fs;
  use std::path::PathBuf;
  use tempfile::tempdir;
  use tracing_appender::rolling::RollingFileAppender;
  use tracing_appender::rolling::Rotation;
  use uuid::Uuid;

  static CONFIG: Lazy<Config> = Lazy::new(|| Config::new().unwrap());

  #[tokio::test]
  async fn test_clone_and_build() {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push(".dosei/doseid/logs");
    let dir = path.parent().unwrap();
    if !dir.exists() {
      let _ = fs::create_dir_all(dir);
    }

    // create file rotation path
    let file_appender = RollingFileAppender::builder()
      .rotation(Rotation::DAILY)
      .filename_prefix("doseid.logs")
      .build(path)
      .expect("Unable to write logs to file, please check permissions.");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();

    let temp_dir = tempdir().expect("Failed to create a temp dir");
    let temp_path = temp_dir.path();

    let repo: anyhow::Result<Repository> =
      git_clone("https://github.com/Alw3ys/dosei-bot.git", temp_path, None).await;
    build(
      Uuid::new_v4(),
      Uuid::new_v4(),
      "test".to_string(),
      temp_path,
    )
    .await;
    drop(temp_dir);
    assert!(repo.is_ok())
  }
}
