use crate::config::{Config, SessionCredentials};
use anyhow::anyhow;
use clap::{Arg, ArgMatches, Command};
use reqwest::blocking::Client;
use serde_json::json;
use std::io;
use std::io::Write;

pub fn command() -> Command {
  Command::new("login")
    .about("Log in to a cluster")
    .arg(Arg::new("username").help("Enter your username"))
}

pub fn login(arg_matches: &ArgMatches, config: &'static Config) -> anyhow::Result<()> {
  let username = if let Some(username) = arg_matches.get_one::<String>("username") {
    username.to_string()
  } else {
    let mut username = String::new();
    print!("Enter your username: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut username)?;
    username.trim().to_string()
  };

  let password = rpassword::prompt_password("Enter your password: ").unwrap();

  let login_url = format!("{}/login", config.api_base_url);
  let body = json!({ "username": username, "password": password });
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
