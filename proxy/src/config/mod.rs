mod default;

use anyhow::Context;
use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub host: String,
  pub port: u16,
  pub ping_port: u16,
  pub database_url: String,
}

impl Config {
  pub fn new() -> anyhow::Result<Config> {
    // Load env variables from `.env`, if any.
    dotenv().ok();

    // Default to RUST_LOG level info
    if env::var("RUST_LOG").is_err() {
      env::set_var("RUST_LOG", "info");
    }

    // Configure logging
    let subscriber = tracing_subscriber::fmt().with_target(false).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(Config {
      host: env::var("DOSEI_PROXY_HOST").unwrap_or(default::HOST.to_string()),
      port: env::var("DOSEI_PROXY_PORT")
        .unwrap_or_else(|_| default::PORT.to_string())
        .parse()
        .context("Invalid port number")?,
      ping_port: env::var("DOSEI_PROXY_PING_PORT")
        .unwrap_or_else(|_| default::PING_PORT.to_string())
        .parse()
        .context("Invalid port number")?,
      database_url: env::var("DATABASE_URL").unwrap_or(default::DATABASE_URL.to_string()),
    })
  }

  pub fn address(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }

  pub fn ping_address(&self) -> String {
    format!("{}:{}", self.host, self.ping_port)
  }
}
