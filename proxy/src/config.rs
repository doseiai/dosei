use anyhow::Context;
use clap::Parser;
use dotenv::dotenv;
use std::fmt::Formatter;
use std::{env, fmt};
use tracing::warn;

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
  pub mongo_url: String,
  pub redis_url: Option<String>,
}

impl Config {
  pub fn new() -> anyhow::Result<Config> {
    dotenv().ok();
    let args = Args::parse();
    if env::var("RUST_LOG").is_err() {
      env::set_var("RUST_LOG", "info");
    }

    // initialise logging
    let subscriber = tracing_subscriber::fmt()
      .with_line_number(true)
      .with_target(true)
      .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let redis_url = match env::var("REDIS_URL") {
      Ok(_url) => {
        warn!("TODO: Implement redis, falling back to single instance caching.");
        None
        // Some(url)
      }
      Err(_) => {
        warn!("Single instance caching in use. Concurrent replicas require REDIS_URL.");
        None
      }
    };
    Ok(Config {
      address: Address {
        host: args.host.clone(),
        port: args.port,
      },
      mongo_url: env::var("MONGODB_URL").context("MONGODB_URL is required.")?,
      redis_url,
    })
  }
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
