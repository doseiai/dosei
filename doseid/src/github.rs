use crate::config::Config;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;

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
  use crate::github::create_github_app_jwt;

  #[test]
  fn test_create_github_app_jwt() {
    let config: &'static Config = Box::leak(Box::new(Config::new().unwrap()));
    let result = create_github_app_jwt(config);
    assert!(result.is_ok());
  }
}
