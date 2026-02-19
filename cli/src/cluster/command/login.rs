use crate::config::{ApiClient, ClusterConfig, Config, SessionCredentials};
use crate::ssh::SSH;
use anyhow::{anyhow, Context};
use std::io;
use std::io::Write;
use std::path::PathBuf;

pub fn command(name: Option<String>, username: Option<String>, yes: bool) -> anyhow::Result<()> {
  let cluster_name = if let Some(name) = name {
    name
  } else {
    let mut input = String::new();
    print!("Enter the cluster name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    input.trim().to_string()
  };

  let username = if let Some(username) = username {
    username.to_string()
  } else {
    let mut username = String::new();
    print!("Enter your username: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut username)?;
    username.trim().to_string()
  };

  let ssh_key_path = if yes {
    let default_path = SSH::get_default_ssh_key_path()
      .context("Failed to get default SSH key path and -y flag was specified")?
      .to_string_lossy()
      .to_string();

    println!("Using default SSH key: {}", default_path);
    default_path
  } else {
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
    cluster_name.clone(),
    ClusterConfig {
      id: None,
      username: username.clone(),
      ssh_key: Some(ssh_key_path.clone()),
    },
  );
  user_config.save()?;

  let login_url = if cluster_name.starts_with("http://") || cluster_name.starts_with("https://") {
    format!("{}/auth/login/ssh", cluster_name)
  } else {
    format!("https://{}/auth/login/ssh", cluster_name)
  };

  let response = ApiClient::default()?
    .post(login_url)
    .bearer_auth(ApiClient::bearer_ssh_token(Some(PathBuf::from(&ssh_key_path)))?)
    .send()?;

  let status_code = response.status();
  if status_code.is_success() {
    let _session = response.json::<SessionCredentials>()?;
    println!("Login Succeeded!");
    return Ok(());
  }
  println!("{}", status_code);
  Err(anyhow!("Login Failed!"))
}
