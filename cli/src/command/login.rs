use crate::config::{Config, SessionCredentials};
use anyhow::anyhow;
use reqwest::blocking::Client;
use serde_json::json;

pub fn login(config: &'static Config) -> anyhow::Result<()> {
  let login_url = format!("{}/login", config.api_base_url);
  let body = json!({ "username": "dosei", "password": "dosei" });

  let response = Client::new().post(login_url).json(&body).send()?;

  let status_code = response.status();
  if status_code.is_success() {
    let session = response.json::<SessionCredentials>()?;
    config.store_token_from_session(&session)?;
    println!("Login Succeeded!");
    return Ok(());
  }
  Err(anyhow!("Login Failed!"))
}
