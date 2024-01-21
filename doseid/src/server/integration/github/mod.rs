pub(crate) mod route;
pub(crate) mod schema;

use crate::server::integration::github::schema::{UserGithub, UserGithubEmail};
use crate::server::integration::{git_clone, git_push};
use anyhow::{anyhow, Context};
use chrono::{Duration, Utc};
use git2::Repository;
use hmac::{Hmac, Mac};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::header::HeaderMap;
use reqwest::{header, Client, Error, Response, StatusCode};
use serde_json::{json, Value};
use sha2::Sha256;
use std::env;
use std::path::Path;
use tracing::{error, warn};

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
    git_clone(
      &self
        .repo_auth_url(repo_full_name, access_token, installation_id)
        .await?,
      to_path,
      branch,
    )
    .await
  }

  pub async fn git_push(
    &self,
    repo_full_name: String,
    from_path: &Path,
    access_token: Option<&str>,
    installation_id: Option<i64>,
    name: &str,
    email: &str,
  ) -> anyhow::Result<()> {
    git_push(
      &self
        .repo_auth_url(repo_full_name, access_token, installation_id)
        .await?,
      from_path,
      name,
      email,
    )
    .await
  }

  pub async fn new_individual_repository(
    &self,
    name: &str,
    private: Option<bool>,
    access_token: &str,
  ) -> Result<Value, CreateRepoError> {
    let response = self
      .github_client()?
      .post("https://api.github.com/user/repos")
      .bearer_auth(access_token)
      .json(&json!({"name": name, "private": private.unwrap_or(true) }))
      .send()
      .await?;

    let status_code = response.status();
    if status_code.is_success() {
      let body = response.json::<Value>().await?;
      return Ok(body);
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

  async fn delete_repository(
    &self,
    repo_full_name: &str,
    access_token: &str,
  ) -> Result<Response, Error> {
    self
      .github_client()?
      .delete(format!("https://api.github.com/repos/{}", repo_full_name))
      .bearer_auth(access_token)
      .send()
      .await?
      .error_for_status()
  }

  pub async fn get_user(&self, access_token: &str) -> Result<UserGithub, Error> {
    let response = self
      .github_client()?
      .get("https://api.github.com/user")
      .bearer_auth(access_token)
      .send()
      .await?;
    let status_code = response.status();
    if status_code.is_success() {
      let mut user = response.json::<UserGithub>().await?;
      user.access_token = Some(access_token.to_string());
      user.emails = Some(self.get_user_emails(access_token).await?);
      return Ok(user);
    }
    Err(response.error_for_status_ref().err().unwrap())
  }

  async fn get_user_emails(&self, access_token: &str) -> Result<Vec<UserGithubEmail>, Error> {
    let response = self
      .github_client()?
      .get("https://api.github.com/user/emails")
      .bearer_auth(access_token)
      .send()
      .await?;
    let status_code = response.status();
    if status_code.is_success() {
      let body = response.json::<Vec<UserGithubEmail>>().await?;
      return Ok(body);
    }
    Err(response.error_for_status_ref().err().unwrap())
  }

  pub async fn get_user_access_token(&self, code: String) -> Result<String, AccessTokenError> {
    let response = self
      .github_client()?
      .post("https://github.com/login/oauth/access_token")
      .json(&json!({
        "client_id": self.client_id,
        "client_secret": self.client_secret,
        "code": code
      }))
      .send()
      .await?;

    let status_code = response.status();
    if status_code.is_success() {
      let body = response.json::<Value>().await?;
      if let Some(access_token) = body.get("access_token").and_then(|v| v.as_str()) {
        return Ok(access_token.to_string());
      }
      if body
        .get("error")
        .map_or(false, |e| e == "bad_verification_code")
      {
        return Err(AccessTokenError::BadVerificationCode);
      }
      error!("AccessTokenError Unhandled Error {:?}", body);
      return Err(AccessTokenError::Unhandled);
    }
    Err(AccessTokenError::Request(
      response.error_for_status_ref().err().unwrap(),
    ))
  }

  async fn repo_auth_url(
    &self,
    repo_full_name: String,
    access_token: Option<&str>,
    installation_id: Option<i64>,
  ) -> anyhow::Result<String> {
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
    Ok(repo_link)
  }

  async fn get_installation_token(&self, installation_id: i64) -> anyhow::Result<String> {
    let url = format!(
      "https://api.github.com/app/installations/{}/access_tokens",
      installation_id
    );
    let jwt = self.create_github_app_jwt()?;
    let response = self
      .github_client()?
      .post(&url)
      .bearer_auth(jwt)
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

  pub fn verify_signature(&self, payload_body: &[u8], signature: &[u8]) -> anyhow::Result<()> {
    let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())?;
    mac.update(payload_body);

    let signature_str = std::str::from_utf8(signature)?;
    let signature_bytes = hex::decode(&signature_str["sha256=".len()..])?;
    mac
      .verify_slice(&signature_bytes)
      .map_err(|_| anyhow!("invalid secret"))
  }

  pub fn github_client(&self) -> Result<Client, Error> {
    let mut headers = HeaderMap::new();
    headers.insert(
      header::ACCEPT,
      header::HeaderValue::from_static("application/vnd.github.v3+json"),
    );
    headers.insert(
      header::USER_AGENT,
      header::HeaderValue::from_static("Dosei"),
    );
    let client = Client::builder().default_headers(headers).build()?;
    Ok(client)
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

#[derive(Debug, thiserror::Error)]
pub enum AccessTokenError {
  #[error("Request failed")]
  Request(#[from] Error),

  #[error("Unhandled")]
  Unhandled,

  #[error("The code passed is incorrect or expired.")]
  BadVerificationCode,
}

#[cfg(test)]
mod tests {
  use crate::test::CONFIG;
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
      .new_individual_repository(
        "rust-tests-create",
        None,
        &env::var("GITHUB_TEST_ACCESS_TOKEN").unwrap(),
      )
      .await;
    assert!(result.is_ok(), "Failed to create repository: {:?}", result);
  }
}
