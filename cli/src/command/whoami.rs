use crate::config::Config;
use clap::Command;
use serde_json::Value;

pub fn command() -> Command {
  Command::new("whoami").about("Display the current logged-in user")
}

pub fn whoami(config: &'static Config) -> anyhow::Result<()> {
  let user_url = format!("{}/user", config.api_base_url);

  let response = config
    .api_client()?
    .get(user_url)
    .bearer_auth(config.bearer_token())
    .send()?;

  let status_code = response.status();
  if status_code.is_success() {
    let body = response.json::<Value>()?;
    println!("{}", body.get("name").and_then(|v| v.as_str()).unwrap());
    return Ok(());
  }
  response.error_for_status()?;
  Ok(())
}
