use crate::config::Config;
use crate::git::git_clone;
use chrono::{Duration, Utc};
use git2::Repository;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

pub async fn github_clone(
  from_url: &str,
  to_path: &Path,
  branch: Option<&str>,
  access_token: Option<&str>,
  installation_id: Option<&str>,
  config: &'static Config,
) -> anyhow::Result<Repository> {
  let github_token = match access_token {
    Some(token) => Some(token.to_string()),
    None => match installation_id {
      Some(id) => Some(get_installation_token(config, id).await?),
      None => None,
    },
  };

  let mut repo_link = from_url.to_string();
  if let Some(token) = &github_token {
    repo_link = repo_link.replace("https://", &format!("https://x-access-token:{}@", token));
  }
  git_clone(&repo_link, to_path, branch).await
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
  use crate::git::github::create_github_app_jwt;
  use git2::Repository;
  use once_cell::sync::Lazy;
  use tempfile::tempdir;

  static CONFIG: Lazy<Config> = Lazy::new(|| Config::new().unwrap());

  #[test]
  fn test_create_github_app_jwt() {
    let result = create_github_app_jwt(&CONFIG);
    assert!(result.is_ok());
  }

  async fn test_clone() {
    let temp_dir = tempdir().expect("Failed to create a temp dir");
    let repo_path = temp_dir.path();

    let repo: anyhow::Result<Repository> = crate::git::github::github_clone(
      "https://github.com/doseiai/dosei.git",
      repo_path,
      None,
      None,
      None,
      &CONFIG,
    )
    .await;
    drop(temp_dir);
    assert!(repo.is_ok())
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
