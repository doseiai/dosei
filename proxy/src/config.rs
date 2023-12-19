use anyhow::Context;
use clap::Parser;
use dotenv::dotenv;
use std::fmt::Formatter;
use std::{env, fmt};

pub fn init() -> anyhow::Result<Config> {
  dotenv().ok();
  let args = Args::parse();
  if env::var("RUST_LOG").is_err() {
    env::set_var("RUST_LOG", "info");
  }
  env_logger::init();
  Ok(Config {
    address: Address {
      host: args.host.clone(),
      port: args.port,
    },
    mongo_uri: env::var("MONGODB_URL").context("MONGODB_URL is required.")?,
  })
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, disable_help_flag = true)]
struct Args {
  #[arg(short, long, default_value = "127.0.0.1")]
  host: String,
  #[arg(short, long, default_value = "8080")]
  port: u16,
  #[arg(short, long)]
  connect: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
  pub address: Address,
  pub mongo_uri: String,
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
