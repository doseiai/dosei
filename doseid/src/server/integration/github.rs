use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use tracing::{error, warn};

pub async fn api_integration_github_events(
  headers: axum::http::HeaderMap,
  Json(payload): Json<Value>,
) -> Result<StatusCode, StatusCode> {
  if let Some(event_type) = headers.get("X-GitHub-Event") {
    match event_type.to_str().unwrap_or("") {
      "ping" => Ok(StatusCode::OK),
      "check_suite" => {
        if let Ok(parsed_payload) = serde_json::from_value::<CheckSuiteHookPayload>(payload) {
          handle_check_suite(parsed_payload);
          Ok(StatusCode::OK)
        } else {
          error!("Failed to parse check_suite payload");
          Ok(StatusCode::INTERNAL_SERVER_ERROR)
        }
      }
      event_name => {
        warn!("Github Event: {} not handled", event_name);
        Ok(StatusCode::OK)
      }
    }
  } else {
    Ok(StatusCode::OK)
  }
}

fn handle_check_suite(payload: CheckSuiteHookPayload) {
  // Push to default branch == PROD Deployment
  if payload.check_suite.head_branch == payload.repository.default_branch {
    // TODO: Trigger build
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
  id: i32,
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
  id: i32,
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
  id: i32,
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
  id: i32,
  node_id: String,
}
