mod app;

use crate::deployment::app::import_dosei_app;
use crate::docker::build_image;
use std::path::Path;

async fn build(folder_path: &Path) {
  let detected_docker_file = dosei_util::package_manager::_resolve_docker(folder_path);
  if detected_docker_file {
    println!("Detected `Dockerfile`");
    let _ = build_image("example", "example", folder_path).await;
    if let Ok(app) = import_dosei_app("example", "example", folder_path).await {
      println!("{:?}", app);
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::deployment::build;
  use crate::git::git_clone;
  use git2::Repository;
  use tempfile::tempdir;

  #[tokio::test]
  async fn test_build() {
    let temp_dir = tempdir().unwrap();
    let repo_path = temp_dir.path();

    let repo: anyhow::Result<Repository> =
      git_clone("https://github.com/Alw3ys/dosei-bot.git", repo_path, None).await;
    build(repo_path).await;
    drop(temp_dir);
  }
}
