use dotenv::dotenv;
use std::fmt::Formatter;
use std::{env, fmt};

pub fn init() -> anyhow::Result<Config> {
  dotenv().ok();
  if env::var("RUST_LOG").is_err() {
    env::set_var("RUST_LOG", "info");
  }
  env_logger::init();
  Ok(Config {
    address: Address {
      host: "127.0.0.1".to_string(),
      port: 8081,
    },
  })
}

#[derive(Debug, Clone)]
pub struct Config {
  pub address: Address,
}

#[derive(Debug, Clone)]
pub struct Address {
  pub host: String,
  pub port: u16,
}

impl fmt::Display for Address {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}", self.host, self.port)
  }
}
