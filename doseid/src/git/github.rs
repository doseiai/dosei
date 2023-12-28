use crate::config::Config;
use crate::git::{get_installation_token, git_clone};
use git2::Repository;
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
