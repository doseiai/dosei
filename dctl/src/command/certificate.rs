use crate::config::Config;
use clap::{Arg, ArgMatches, Command};
use serde_json::json;

pub fn subcommand() -> Command {
  Command::new("certificate")
    .about("Certificates commands")
    .subcommand_required(true)
    .subcommand(
      Command::new("new")
        .about("New certificate")
        .arg(Arg::new("name").index(1).required(true)),
    )
}

pub fn new_certificate(config: &'static Config, arg_matches: &ArgMatches) {
  let name = arg_matches.get_one::<String>("name").expect("required");
  let response = config
    .cluster_api_client()
    .expect("Client connection failed")
    .post(format!("{}/certificate", config.api_base_url))
    .json(&json!({"domain_name": name}))
    .bearer_auth(config.bearer_token())
    .send()
    .unwrap();
  if response.status().is_success() {
    println!(
      "
    {name}

    Set the following record on your DNS provider to continue:

    Type Name Value
    A    @    34.175.93.38
    "
    );
  }
}
