mod github;
use crate::config::Config;
use anyhow::Context;
use chrono::{Duration, Utc};
use git2::build::RepoBuilder;
use git2::{FetchOptions, Repository};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::path::Path;
use tokio::task;
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
      info!("branch provided");
      repo_builder.branch(&branch_name);
    }
    match repo_builder.clone(&from_url, &to_path) {
      Ok(repo) => Ok(repo),
      Err(e) => panic!("failed to clone: {}", e),
    }
  })
  .await?
}

async fn update_deployment_status(
  config: &'static Config,
  status: GithubDeploymentStatus,
  repo_full_name: &str,
  installation_id: &str,
  commit_id: &str,
) -> Result<(), reqwest::Error> {
  let github_token = get_installation_token(config, installation_id).await;
  let github_api_repo_url = format!("https://api.github.com/repos/{}", repo_full_name);
  let client = Client::new();
  let headers = header::HeaderMap::new();

  let status_info = status.info();

  client
    .post(format!("{}/statuses/{}", github_api_repo_url, commit_id))
    .json(&json!({
        "state": status_info.state,
        "description": status_info.message,
        "target_url": format!("{}/{}/deployments/{}", "app_base_url", repo_full_name, commit_id),
        "context": config.github_integration.app_name
    }))
    .headers(headers.clone())
    .send()
    .await?;

  if status == GithubDeploymentStatus::Succeeded {
    client
      .post(format!(
        "{}/commits/{}/comments",
        github_api_repo_url, commit_id
      ))
      .json(&json!({
          "body": "Successfully deployed in production."
      }))
      .headers(headers)
      .send()
      .await?;
  }
  Ok(())
}

async fn get_installation_token(
  config: &'static Config,
  installation_id: &str,
) -> anyhow::Result<String> {
  let url = format!(
    "https://api.github.com/app/installations/{}/access_tokens",
    installation_id
  );
  let jwt = create_github_app_jwt(config)?;

  let response = Client::new()
    .post(&url)
    .bearer_auth(jwt)
    .header("Accept", "application/vnd.github.v3+json")
    .send()
    .await?;

  let json: Value = response.json().await?;
  Ok(json["token"].to_string())
}

pub fn create_github_app_jwt(config: &'static Config) -> anyhow::Result<String> {
  let now = Utc::now();
  let expiration = now + Duration::minutes(10);

  let claims = json!({
      "iat": now.timestamp(),
      "exp": expiration.timestamp(),
      "iss": config.github_integration.app_id
  });

  let encoding_key = EncodingKey::from_rsa_pem(config.github_integration.private_key.as_bytes())?;

  Ok(encode(
    &Header::new(Algorithm::RS256),
    &claims,
    &encoding_key,
  )?)
}

#[derive(PartialEq)]
enum GithubDeploymentStatus {
  Deploying,
  Succeeded,
}

#[derive(Serialize, Deserialize, PartialEq)]
struct DeploymentInfo {
  state: String,
  message: String,
}

impl GithubDeploymentStatus {
  fn info(&self) -> DeploymentInfo {
    match self {
      GithubDeploymentStatus::Deploying => DeploymentInfo {
        state: "pending".to_string(),
        message: "Deploying...".to_string(),
      },
      GithubDeploymentStatus::Succeeded => DeploymentInfo {
        state: "success".to_string(),
        message: "Deployment succeeded".to_string(),
      },
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::config::Config;
  use crate::git::create_github_app_jwt;
  use git2::Repository;
  use once_cell::sync::Lazy;
  use tempfile::tempdir;
  use tokio::fs;
  use tracing::info;

  static CONFIG: Lazy<Config> = Lazy::new(|| Config::new().unwrap());

  #[test]
  fn test_create_github_app_jwt() {
    let result = create_github_app_jwt(&CONFIG);
    assert!(result.is_ok());
  }

  async fn test_clone() {
    let temp_dir = tempdir().expect("Failed to create a temp dir");
    let dir = temp_dir.path().to_owned();
    let repo_path = temp_dir.path();

    let repo: Repository = crate::git::github::github_clone(
      "https://github.com/doseiai/dosei.git",
      repo_path,
      None,
      None,
      None,
      &CONFIG,
    )
    .await
    .unwrap();

    info!("Temp directory: {:?}", repo_path);

    let mut paths = fs::read_dir(&dir).await.expect("Failed to read dir");
    while let Some(path) = paths.next_entry().await.expect("Failed to read next entry") {
      info!("Path: {:?}", path.path());
    }
    drop(temp_dir)
  }

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
}
