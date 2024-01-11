use git2::Repository;
use std::path::Path;
pub(crate) mod github;

// TODO: Design concept of Git Source Integration trait
pub trait GitSource {
  fn new() -> anyhow::Result<()>;

  fn git_pull(from_url: &str, to_path: &Path, branch: Option<&str>) -> anyhow::Result<Repository>;

  fn git_push(from_url: &str, from_path: &Path, name: &str, email: &str) -> anyhow::Result<()>;
}
