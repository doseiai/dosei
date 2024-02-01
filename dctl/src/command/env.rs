use crate::config::Config;
use crate::session::get_session_user;
use clap::Command;
use serde::{Deserialize, Serialize};

pub fn sub_command() -> Command {
  Command::new("env")
    .about("Environment variables commands")
    .subcommand_required(true)
    .subcommand(Command::new("list").about("List environment variables"))
    .subcommand(Command::new("set").about("Set environment variables"))
}

pub fn list_env(config: &'static Config) {
  let user = match get_session_user(config) {
    Ok(user) => user,
    _ => panic!("Something went wrong"),
  };
  let response = config
    .cluster_api_client()
    .expect("Client connection failed")
    .get(format!("{}/envs/{}", config.api_base_url, user.id))
    .bearer_auth(config.bearer_token())
    .send()
    .unwrap();
  if response.status().is_success() {
    let result = response.json::<Vec<Env>>().unwrap();
    for env in result {
      println!("{}={}", env.name, env.value);
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct Env {
  name: String,
  value: String,
}

pub fn set_env(_config: &'static Config) {
  todo!();
}
