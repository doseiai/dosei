use crate::config::Config;
use crate::deployment::build_from_github;
use crate::git::github::GithubIntegration;
use axum::http::StatusCode;
use axum::Extension;
use serde::Deserialize;
use serde_json::Value;
use tracing::{error, warn};

pub async fn api_integration_github_events(
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  body: axum::body::Bytes,
) -> Result<StatusCode, StatusCode> {
  let github_integration = match config.github_integration.as_ref() {
    Some(github) => github,
    None => {
      error!("Github integration not enabled");
      return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
  };
  if let Some(signature_header) = headers.get("X-Hub-Signature-256") {
    if let Err(err) = github_integration.verify_signature(&body, signature_header.as_bytes()) {
      error!("{}", err);
      return Err(StatusCode::FORBIDDEN);
    }
  } else {
    return Err(StatusCode::UNAUTHORIZED);
  }

  let payload: Value = match serde_json::from_slice(&body) {
    Ok(val) => val,
    Err(_) => return Err(StatusCode::BAD_REQUEST),
  };

  if let Some(event_type) = headers.get("X-GitHub-Event") {
    match event_type.to_str().unwrap_or("") {
      "ping" => Ok(StatusCode::OK),
      "check_suite" => match serde_json::from_value::<CheckSuiteHookPayload>(payload) {
        Ok(parsed_payload) => {
          handle_check_suite(github_integration, parsed_payload);
          Ok(StatusCode::OK)
        }
        Err(e) => {
          error!("Failed to parse check_suite payload: {:?}", e);
          Ok(StatusCode::INTERNAL_SERVER_ERROR)
        }
      },
      event_name => {
        warn!("Github Event: {} not handled", event_name);
        Ok(StatusCode::OK)
      }
    }
  } else {
    Ok(StatusCode::OK)
  }
}

fn handle_check_suite(github_integration: &GithubIntegration, payload: CheckSuiteHookPayload) {
  // Push to default branch == PROD Deployment
  if payload.check_suite.head_branch == payload.repository.default_branch {
    // Clone the necessary data for 'static lifetime
    let github_integration_clone = github_integration.clone();
    let head_sha = payload.check_suite.head_sha.clone();
    let full_name = payload.repository.full_name.clone();
    let installation_id = payload.installation.id;

    tokio::spawn(async move {
      build_from_github(
        github_integration_clone,
        head_sha,
        full_name,
        installation_id,
      )
      .await;
    });
  }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct CheckSuiteHookPayload {
  check_suite: CheckSuite,
  repository: Repository,
  sender: NamedUser,
  installation: Installation,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct CheckSuite {
  id: i64,
  node_id: String,
  head_branch: String,
  head_sha: String,
  status: String,
  conclusion: Option<String>,
  url: String,
  before: String,
  after: String,
  // TODO: pull_requests: Vec<HashMap<String, serde_json::Value>>,
  head_commit: HeadCommit,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct HeadCommit {
  id: String,
  message: String,
  author: GitCommitAuthor,
  committer: GitCommitAuthor,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct GitCommitAuthor {
  name: String,
  email: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct NamedUser {
  login: String,
  id: i64,
  avatar_url: String,
  url: String,
  html_url: String,
  followers_url: String,
  following_url: String,
  gists_url: String,
  starred_url: String,
  subscriptions_url: String,
  organizations_url: String,
  repos_url: String,
  events_url: String,
  received_events_url: String,
  #[serde(rename = "type")]
  type_field: String,
  site_admin: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Repository {
  id: i64,
  name: String,
  full_name: String,
  description: Option<String>,
  #[serde(default)]
  topics: Vec<String>,
  visibility: String,
  forks: i32,
  open_issues: i32,
  watchers: i32,
  default_branch: String,
  html_url: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Installation {
  id: i64,
  node_id: String,
}
