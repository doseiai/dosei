use crate::cli::Cli;
use crate::config::{ClusterConfig, Config, SessionCredentials};
use crate::ssh::SSH;
use anyhow::{anyhow, Context};
use reqwest::blocking::Client;
use serde_json::json;
use std::io;
use std::io::Write;

pub fn command(name: Option<String>, username: Option<String>, yes: bool) -> anyhow::Result<()> {
  let cluster = Cli::get_default_cluster_or_ask(name)?;

  let username = if let Some(username) = username {
    username.to_string()
  } else {
    let mut username = String::new();
    print!("Enter your username: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut username)?;
    username.trim().to_string()
  };

  let _ssh_key_path = if yes {
    // If -y is provided, automatically use default SSH key or fail
    let default_path = SSH::get_default_ssh_key_path()
      .context("Failed to get default SSH key path and -y flag was specified")?
      .to_string_lossy()
      .to_string();

    println!("Using default SSH key: {}", default_path);
    default_path
  } else {
    // Original interactive flow
    let default_ssh_key_path = SSH::get_default_ssh_key_path()
      .context("Failed to get default ssh key path. Define one")?
      .to_string_lossy()
      .to_string();

    println!(
      "Would you like to use your default SSH key ({})?",
      default_ssh_key_path
    );
    print!("Enter [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
      default_ssh_key_path
    } else {
      let mut custom_path = String::new();
      print!("Enter the path to your SSH key: ");
      io::stdout().flush()?;
      io::stdin().read_line(&mut custom_path)?;
      custom_path.trim().to_string()
    }
  };

  let mut user_config = Config::load()?;
  user_config.add_cluster(
    cluster.0.clone(),
    ClusterConfig {
      id: None,
      username: username.clone(),
      ssh_key: None,
    },
  );
  user_config.save()?;

  let login_url = format!("{}/auth/login", cluster.0);
  let body = json!({ "username": username });
  let response = Client::new().post(login_url).json(&body).send()?;

  let status_code = response.status();
  if status_code.is_success() {
    let _session = response.json::<SessionCredentials>()?;
    println!("Login Succeeded!");
    return Ok(());
  }
  println!("{}", status_code);
  Err(anyhow!("Login Failed!"))
}
