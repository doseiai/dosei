use crate::config::Config;
use crate::session::get_session_user;
use clap::ArgMatches;
use serde_json::{json, Value};

pub fn new_certificate(config: &'static Config, arg_matches: &ArgMatches) {
  let name = arg_matches.get_one::<String>("name").expect("required");
  let user = match get_session_user(config) {
    Ok(user) => user,
    _ => panic!("Something went wrong"),
  };
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
