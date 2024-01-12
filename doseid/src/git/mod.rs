pub(crate) mod github;
use git2::build::RepoBuilder;
use git2::{FetchOptions, IndexAddOption, Repository, Signature};
use regex::Regex;
use std::path::Path;
use tokio::task;
use tokio::time::Instant;
use tracing::info;

pub async fn git_clone(
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

pub async fn git_push(
  from_url: &str,
  from_path: &Path,
  name: &str,
  email: &str,
) -> anyhow::Result<()> {
  let from_url = from_url.to_string();
  let from_path = from_path.to_path_buf();
  let name = name.to_string();
  let email = email.to_string();
  task::spawn_blocking(move || {
    let repo = Repository::init(from_path)?;

    let sig = Signature::now(&name, &email)?;

    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let oid = index.write_tree()?;

    let tree = repo.find_tree(oid)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    let mut remote = repo.remote("origin", &from_url)?;
    remote.push(&["refs/heads/main:refs/heads/main"], None)?;
    Ok(())
  })
  .await?
}

#[cfg(test)]
mod tests {
  use crate::git::git_clone;
  use git2::Repository;
  use tempfile::tempdir;

  #[tokio::test]
  async fn test_clone_repos() {
    use futures::future::join_all;

    let tests: Vec<_> = (0..10)
      .map(|_| {
        tokio::spawn(async {
          test_clone().await;
        })
      })
      .collect();

    join_all(tests).await;
  }

  async fn test_clone() {
    let temp_dir = tempdir().expect("Failed to create a temp dir");
    let repo_path = temp_dir.path();

    let repo: anyhow::Result<Repository> =
      git_clone("https://github.com/Alw3ys/dosei-bot.git", repo_path, None).await;
    drop(temp_dir);
    assert!(repo.is_ok())
  }
}
