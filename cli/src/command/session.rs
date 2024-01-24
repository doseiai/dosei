use crate::config::Config;
use crate::session::get_session_user;

pub fn session(config: &'static Config) {
  println!("Cluster Host: {}", config.api_base_url);
  if let Some(current_session) = config.session() {
    println!("Session ID: {}", current_session.id);
    if let Ok(user) = get_session_user(config) {
      println!("User: {} ({})", user.username, user.email)
    }
  }
}
