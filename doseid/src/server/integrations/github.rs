use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn};

pub async fn api_integration_github_events(
  headers: axum::http::HeaderMap,
  Json(payload): Json<Value>,
) -> Result<StatusCode, StatusCode> {
  if let Some(event_type) = headers.get("X-GitHub-Event") {
    match event_type.to_str().unwrap_or("") {
      "ping" => Ok(StatusCode::OK),
      "check_suite" => {
        info!("{}", payload);
        Ok(StatusCode::OK)
      }
      _ => {
        warn!(
          "Github Event: {} not handled",
          event_type.to_str().unwrap_or("")
        );
        Ok(StatusCode::OK)
      }
    }
  } else {
    Ok(StatusCode::OK)
  }
}

struct CheckSuiteHookPayload {
  repository: Repository,
  sender: NamedUser,
  installation: Installation,
}

#[derive(Debug, Serialize, Deserialize)]
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
  // pull_requests: Vec<HashMap<String, serde_json::Value>>,
  head_commit: HeadCommit,
}

#[derive(Debug, Serialize, Deserialize)]
struct HeadCommit {
  id: String,
  message: String,
  author: GitCommitAuthor,
  committer: GitCommitAuthor,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitCommitAuthor {
  name: String,
  email: String,
}

#[derive(Serialize, Deserialize)]
pub struct NamedUser {
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

#[derive(Serialize, Deserialize)]
pub struct Repository {
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

#[derive(Serialize, Deserialize)]
struct Installation {
  id: i32,
  node_id: String,
}
