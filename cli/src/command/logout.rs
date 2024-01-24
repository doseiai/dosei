use crate::config::Config;
use reqwest::StatusCode;
use serde_json::Value;

pub fn logout(config: &'static Config) {
  let response = config
    .cluster_api_client()
    .expect("Client connection failed")
    .delete(format!("{}/auth/logout", config.api_base_url))
    .query(&[("session_id", config.session().unwrap().id)])
    .bearer_auth(config.bearer_token())
    .send()
    .unwrap();
  let status_code = response.status();
  if status_code.is_success() {
    let body = response.json::<Value>().unwrap();
    config.remove_stored_credentials().unwrap();
    return println!("{}", body.get("message").unwrap());
  }
  if status_code == StatusCode::NOT_FOUND {
    return eprintln!("Session not found");
  }
  response.error_for_status().unwrap();
}
