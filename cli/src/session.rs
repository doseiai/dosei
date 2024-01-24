use crate::config::Config;
use reqwest::Error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn get_session_user(config: &'static Config) -> anyhow::Result<User, Error> {
  let response = config
    .cluster_api_client()
    .expect("Client connection failed")
    .get(format!("{}/user", config.api_base_url))
    .bearer_auth(config.bearer_token())
    .send()?;
  if response.status().is_success() {
    let user = response.json::<User>()?;
    return Ok(user);
  }
  let error = response.error_for_status_ref().err().unwrap();
  Err(error)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
  pub id: Uuid,
  pub username: String,
  pub email: String,
}
