pub mod github;
use git2::build::RepoBuilder;
use git2::{FetchOptions, Repository};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::task;
use tokio::time::Instant;
use tracing::info;

async fn git_clone(
  from_url: &str,
  to_path: &Path,
  branch: Option<&str>,
) -> anyhow::Result<Repository> {
  let from_url = from_url.to_string();
  let to_path = to_path.to_path_buf();
  let branch = branch.map(|s| s.to_string());
  task::spawn_blocking(move || {
    let mut fetch_options = FetchOptions::new();
    fetch_options.depth(1);
    let mut repo_builder = RepoBuilder::new();
    repo_builder.fetch_options(fetch_options);
    if let Some(branch_name) = branch {
      repo_builder.branch(&branch_name);
    }

    let re = Regex::new(r"(?:https?://)?(?:[^@]+@)?([^/]+/[^/]+/[^/.]+)").unwrap();
    match re.captures(&from_url.to_string()) {
      Some(cap) => info!("Cloning {}", &cap[1]),
      None => info!("Cloning repository"),
    }
    let start = Instant::now();
    match repo_builder.clone(&from_url, &to_path) {
      Ok(repo) => {
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        info!("Cloning completed: {:.2}ms", elapsed);
        Ok(repo)
      }
      Err(e) => panic!("failed to clone: {}", e),
    }
  })
  .await?
}
