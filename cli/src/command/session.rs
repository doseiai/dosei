use crate::config::Config;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn session(config: &'static Config) {
  println!("Cluster Host: {}", config.api_base_url);
  if let Some(current_session) = config.session() {
    println!("Session ID: {}", current_session.id);
    let response = config
      .cluster_api_client()
      .expect("Client connection failed")
      .get(format!("{}/user", config.api_base_url))
      .bearer_auth(config.bearer_token())
      .send()
      .unwrap();
    if response.status().is_success() {
      let user = response.json::<User>().unwrap();
      println!("User: {} ({})", user.username, user.email);
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
  id: Uuid,
  username: String,
  email: String,
}
