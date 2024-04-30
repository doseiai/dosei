use crate::config::Config;
use clap::Command;
use reqwest::blocking::multipart;
use std::path::Path;
use std::time::Duration;
use std::{env, fs};

pub fn command() -> Command {
  Command::new("deploy").about("Deploy Dosei App")
}

pub fn deploy(config: &'static Config) -> anyhow::Result<()> {
  let current_dir = env::current_dir()?;
  let path = Path::new(&current_dir);

  dosei_util::dosei_service_config(path)?;

  let output_path = path.join(".dosei/output.tar.gz");
  if let Some(dosei_dir) = output_path.parent() {
    fs::create_dir_all(dosei_dir)?;
  }

  dosei_util::write_tar_gz(path, &output_path)?;

  let deploy_url = format!("{}/deploy", config.api_base_url);
  let body = multipart::Form::new().file("file", output_path)?;

  let response = config
    .api_client()?
    .post(deploy_url)
    .multipart(body)
    .timeout(Duration::from_secs(3600))
    .bearer_auth(config.bearer_token())
    .send()?;

  let status_code = response.status();
  if status_code.is_success() {
    println!("Successfully deployed");
    return Ok(());
  }
  response.error_for_status()?;
  Ok(())
}
