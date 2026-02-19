mod default;

use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeMode {
  Main,
  Worker,
}

#[derive(Debug)]
pub struct Config {
  pub host: String,
  pub port: u16,
  pub database_url: String,
  pub mode: NodeMode,
  pub main_url: Option<String>,
}

impl Config {
  pub fn new() -> anyhow::Result<Config> {
    // Load env variables from `.env`, if any.
    dotenv().ok();

    // Configure logging
    let subscriber = tracing_subscriber::fmt()
      .with_target(false)
      .with_max_level(tracing::Level::INFO)
      .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let port = env::var("DOSEID_PORT")
      .ok()
      .and_then(|p| p.parse().ok())
      .unwrap_or(8080u16);

    let mode = match env::var("DOSEID_MAIN_URL") {
      Ok(_) => NodeMode::Worker,
      Err(_) => NodeMode::Main,
    };

    let main_url = env::var("DOSEID_MAIN_URL").ok();

    Ok(Config {
      host: "0.0.0.0".to_string(),
      port,
      database_url: env::var("DATABASE_URL").unwrap_or(default::DATABASE_URL.to_string()),
      mode,
      main_url,
    })
  }

  pub fn address(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }

  pub fn is_main(&self) -> bool {
    self.mode == NodeMode::Main
  }
}
