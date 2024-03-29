use crate::config::Config;
use clap::Command;
use serde::{Deserialize, Serialize};

pub fn sub_command() -> Command {
  Command::new("token")
    .about("Tokens commands")
    .subcommand_required(true)
    .subcommand(Command::new("list").about("List tokens"))
}

pub fn list_token(config: &'static Config) {
  let response = config
    .cluster_api_client()
    .expect("Client connection failed")
    .get(format!("{}/tokens", config.api_base_url))
    .bearer_auth(config.bearer_token())
    .send()
    .unwrap();
  if response.status().is_success() {
    let tokens = response.json::<Vec<Token>>().unwrap();
    for token in tokens {
      println!("{}={}", token.name, token.value);
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct Token {
  name: String,
  value: String,
}
