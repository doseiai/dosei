use crate::config::Config;
use clap::{Arg, ArgMatches, Command};
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::io::{BufRead, BufReader};

pub fn command() -> Command {
  Command::new("set")
    .about("Set environment variables")
    .arg(
      Arg::new("name")
        .help("The env variable name")
        .index(1)
        .required(true),
    )
    .arg(Arg::new("value").help("The env variable value").index(2))
}

pub fn set_env(arg_matches: &ArgMatches, config: &'static Config) -> anyhow::Result<()> {
  let name = arg_matches.get_one::<String>("name").unwrap();
  let mut value = String::new();
  if dosei_util::secret::is_secret_env(name) {
    value = rpassword::prompt_password("Enter the secret value: ")?;

    let login_url = format!("{}/secret", config.api_base_url);
    let body = json!({ "name": name.clone(), "value": value.clone() });
    let response = Client::new()
      .post(login_url)
      .json(&body)
      .bearer_auth(config.bearer_token())
      .send()?;
    let status_code = response.status();
    if status_code.is_success() {
      let secret = response.json::<Value>()?;
      value = secret.get("value").unwrap().to_string();
    } else {
      response.error_for_status()?;
    }
  } else if let Some(input_value) = arg_matches.get_one::<String>("value") {
    value = input_value.to_string()
  } else {
    print!("Enter the environment variable value: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut value)?;
    value = value.trim().to_string();
  }

  let path = ".env";
  let file = File::open(path);

  let mut env_vars = BTreeMap::new();

  if let Ok(file) = file {
    let reader = BufReader::new(file);
    for line in reader.lines() {
      let line = line?;
      if let Some((key, value)) = line.split_once('=') {
        env_vars.insert(key.trim().to_string(), value.trim().to_string());
      }
    }
  }

  env_vars.insert(name.to_string(), value.to_string());

  let mut file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(path)?;

  for (key, value) in env_vars {
    writeln!(file, "{}={}", key, value)?;
  }

  Ok(())
}
