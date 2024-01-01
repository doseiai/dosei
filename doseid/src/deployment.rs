use crate::docker::{build_image, push_image, run_command};
use dosei_util::Framework;
use std::path::Path;
use tracing::error;

// This is some code I've on python which I need to move here
// app_json = DockerManager.run_command(
// tag,
// f'sh -c "dosei export {app_instance} && cat .dosei/app.json"'
// )
// dosei_app = Dosei(**json.loads(app_json))
// cron_jobs = dosei_app.cron_jobs

async fn build(folder_path: &Path) {
  let detected_docker_file = dosei_util::package_manager::_resolve_docker(folder_path);
  if detected_docker_file {
    println!("Detected `Dockerfile`");
    let _ = build_image("example", "example", folder_path).await;

    match dosei_util::_find_framework_init(&Framework::Dosei, folder_path) {
      Ok(entrypoint) => {
        println!("{}", entrypoint);
        let _ = run_command("example", "example", entrypoint.as_str()).await;
      }
      Err(_) => {
        eprintln!("We couldn't find it!");
        return error!("We couldn't find it");
      }
    }

    // let _ = push_image("example", "example").await;
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
