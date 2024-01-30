use crate::config::Config;
use clap::ArgMatches;
use serde_json::{json, Value};

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
    let result = response.json::<Value>().unwrap();
    println!("{}", result);
  }
}
