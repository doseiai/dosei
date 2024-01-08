use crate::git::git_clone;
use anyhow::{anyhow, Context};
use chrono::{Duration, Utc};
use git2::Repository;
use hmac::{Hmac, Mac};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::{header, Client, Error, Response, StatusCode};
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

  pub async fn new_individual_repo(
    &self,
    name: &str,
    private: Option<bool>,
    access_token: &str,
  ) -> Result<Response, CreateRepoError> {
    let response = Client::new()
      .post("https://api.github.com/user/repos")
      .bearer_auth(access_token)
      .json(&json!({"name": name, "private": private.unwrap_or(true) }))
      .header("Accept", "application/vnd.github.v3+json")
      .header("User-Agent", "Dosei")
      .send()
      .await?;

    let status_code = response.status();
    if status_code.is_success() {
      return Ok(response);
    }

    let error_result = response.error_for_status_ref().err().unwrap(); // safe to unwrap after checking success
    if status_code == StatusCode::UNPROCESSABLE_ENTITY {
      let body = response.json::<Value>().await;
      if let Err(e) = body {
        return Err(CreateRepoError::RequestError(e));
      }
      let json_result = body.unwrap();
      if let Some(errors) = json_result["errors"].as_array() {
        for error in errors {
          if error["message"] == "name already exists on this account" {
            return Err(CreateRepoError::RepoExists);
          }
        }
      }
    }
    Err(CreateRepoError::RequestError(error_result))
  }

  async fn update_deployment_status(
    &self,
    status: GithubDeploymentStatus,
    repo_full_name: &str,
    installation_id: i64,
    commit_id: &str,
  ) -> Result<(), Error> {
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

#[derive(Debug, thiserror::Error)]
pub enum CreateRepoError {
  #[error("Request failed")]
  RequestError(#[from] Error),

  #[error(
    "The specified name is already used for a different Git repository. Please enter a new one."
  )]
  RepoExists,
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
  use crate::test_utils::CONFIG;
  use std::env;

  #[test]
  fn test_create_github_app_jwt() {
    if CONFIG.github_integration.is_some() {
      let result = CONFIG
        .github_integration
        .as_ref()
        .unwrap()
        .create_github_app_jwt();
      assert!(result.is_ok());
    }
  }

  #[tokio::test]
  async fn test_create_repo() {
    let result = CONFIG
      .github_integration
      .as_ref()
      .unwrap()
      .new_individual_repo(
        "rust-tests-create",
        None,
        &env::var("GITHUB_TEST_ACCESS_TOKEN").unwrap(),
      )
      .await;
    assert!(result.is_ok(), "Failed to create repository: {:?}", result)
  }
}
