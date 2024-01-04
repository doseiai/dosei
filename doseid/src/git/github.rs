use crate::git::git_clone;
use anyhow::{anyhow, Context};
use chrono::{Duration, Utc};
use git2::Repository;
use hmac::{Hmac, Mac};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;
use std::env;
use std::path::Path;
use tracing::warn;

type HmacSha256 = Hmac<Sha256>;

pub struct GithubIntegration {
  pub app_name: String,
  pub app_id: String,
  pub client_id: String,
  pub client_secret: String,
  pub private_key: String,
  pub webhook_secret: String,
}

impl GithubIntegration {
  pub fn new() -> anyhow::Result<GithubIntegration> {
    let github_integration = GithubIntegration {
      app_name: env::var("GITHUB_APP_NAME").unwrap_or("Dosei".to_string()),
      app_id: env::var("GITHUB_APP_ID").context("GITHUB_APP_ID is required.")?,
      client_id: env::var("GITHUB_CLIENT_ID").context("GITHUB_CLIENT_ID is required.")?,
      client_secret: env::var("GITHUB_CLIENT_SECRET")
        .context("GITHUB_CLIENT_SECRET is required.")?,
      private_key: env::var("GITHUB_PRIVATE_KEY").context("GITHUB_PRIVATE_KEY is required.")?,
      webhook_secret: env::var("GITHUB_WEBHOOK_SECRET")
        .context("GITHUB_WEBHOOK_SECRET is required.")?,
    };
    warn!("[Integrations] Enabling github.unstable");
    Ok(github_integration)
  }

  pub async fn github_clone(
    &self,
    repo_full_name: String,
    to_path: &Path,
    branch: Option<&str>,
    access_token: Option<&str>,
    installation_id: Option<i64>,
  ) -> anyhow::Result<Repository> {
    let github_token = match access_token {
      Some(token) => Some(token.to_string()),
      None => match installation_id {
        Some(id) => Some(self.get_installation_token(id).await?),
        None => None,
      },
    };

    let mut repo_link = format!("https://github.com/{}", repo_full_name);
    if let Some(token) = &github_token {
      repo_link = repo_link.replace("https://", &format!("https://x-access-token:{}@", token));
    }
    git_clone(&repo_link, to_path, branch).await
  }

  pub fn verify_signature(&self, payload_body: &[u8], signature: &[u8]) -> anyhow::Result<()> {
    let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())?;
    mac.update(payload_body);

    let signature_str = std::str::from_utf8(signature)?;
    let signature_bytes = hex::decode(&signature_str["sha256=".len()..])?;
    mac
      .verify_slice(&signature_bytes)
      .map_err(|_| anyhow!("invalid secret"))
  }

  async fn update_deployment_status(
    &self,
    status: GithubDeploymentStatus,
    repo_full_name: &str,
    installation_id: i64,
    commit_id: &str,
  ) -> Result<(), reqwest::Error> {
    let github_token = self.get_installation_token(installation_id).await;
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
          "context": self.app_name
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

  async fn get_installation_token(&self, installation_id: i64) -> anyhow::Result<String> {
    let url = format!(
      "https://api.github.com/app/installations/{}/access_tokens",
      installation_id
    );
    let jwt = self.create_github_app_jwt()?;

    let response = Client::new()
      .post(&url)
      .bearer_auth(jwt)
      .header("Accept", "application/vnd.github.v3+json")
      .header("User-Agent", "Dosei")
      .send()
      .await?;

    let json: Value = response.json().await?;
    Ok(json["token"].to_string())
  }

  fn create_github_app_jwt(&self) -> anyhow::Result<String> {
    let now = Utc::now();
    let expiration = now + Duration::minutes(10);

    let claims = json!({
        "iat": now.timestamp(),
        "exp": expiration.timestamp(),
        "iss": self.app_id
    });

    let encoding_key = EncodingKey::from_rsa_pem(self.private_key.as_bytes())?;

    Ok(encode(
      &Header::new(Algorithm::RS256),
      &claims,
      &encoding_key,
    )?)
  }
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

// TODO: Support passing settings to run github tests
#[cfg(test)]
mod tests {
  // use crate::config::Config;
  use crate::git::git_clone;
  use git2::Repository;
  // use once_cell::sync::Lazy;
  use tempfile::tempdir;
  //
  //   static CONFIG: Lazy<Config> = Lazy::new(|| Config::new().unwrap());
  //
  //   #[test]
  //   fn test_create_github_app_jwt() {
  //     if CONFIG.github_integration.is_some() {
  //       let result = CONFIG
  //         .github_integration
  //         .as_ref()
  //         .unwrap()
  //         .create_github_app_jwt();
  //       assert!(result.is_ok());
  //     }
  //   }
  //
  async fn test_clone() {
    let temp_dir = tempdir().expect("Failed to create a temp dir");
    let repo_path = temp_dir.path();

    let repo: anyhow::Result<Repository> =
      git_clone("https://github.com/Alw3ys/dosei-bot.git", repo_path, None).await;
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
