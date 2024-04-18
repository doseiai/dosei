use crate::config::Config;
use anyhow::anyhow;
use reqwest::StatusCode;
use serde_json::Value;

pub fn logout(config: &'static Config) -> anyhow::Result<()> {
  let logout_url = format!("{}/logout", config.api_base_url);

  let session = config
    .session()
    .ok_or(anyhow!("You are already logged out"))?;

  let response = config
    .api_client()?
    .delete(logout_url)
    .query(&[("session_id", session.id)])
    .bearer_auth(config.bearer_token())
    .send()?;

  let status_code = response.status();
  if status_code.is_success() {
    let body = response.json::<Value>()?;
    config.remove_stored_credentials()?;
    println!("{}", body.get("message").unwrap());
    return Ok(());
  }
  if status_code == StatusCode::NOT_FOUND {
    return Err(anyhow!("Session not found"));
  }
  response.error_for_status()?;
  Ok(())
}
