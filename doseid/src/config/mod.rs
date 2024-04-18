mod defaults;
mod registry;

use anyhow::Context;
use clap::Parser;
use dotenv::dotenv;
use rand::distributions::Alphanumeric;
use rand::Rng;
use serde::Deserialize;
use serde_json::Value;
use sqlx::{Pool, Postgres};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::{env, fs};
use tracing::warn;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, disable_help_flag = true)]
struct Args {
  #[arg(long, action = clap::ArgAction::Help, help = "Print help")]
  help: Option<bool>,
  #[arg(long, help = "Path to dosei daemon config folder")]
  config: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
  pub host: String,
  pub port: u16,
  pub database_url: String,
  pub jwt_secret: String,
  pub disable_telemetry: bool,
  pub dosei_password: String,
}

impl Config {
  pub fn new() -> anyhow::Result<Config> {
    let args = Args::parse();

    // Load env variables from `.env`, if any.
    dotenv().ok();

    // Default to RUST_LOG level info
    if env::var("RUST_LOG").is_err() {
      env::set_var("RUST_LOG", "info");
    }

    // Configure logging
    let subscriber = tracing_subscriber::fmt().with_target(false).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    if args.config.is_some() {
      let result = config_from_file(Path::new(&args.config.unwrap()));
      println!("{:?}", result);
    }

    Ok(Config {
      host: env::var("DOSEID_HOST").unwrap_or(defaults::HOST.to_string()),
      port: env::var("DOSEID_PORT")
        .unwrap_or_else(|_| defaults::PORT.to_string())
        .parse()
        .context("Invalid port number")?,
      database_url: env::var("DATABASE_URL").unwrap_or(defaults::DATABASE_URL.to_string()),
      dosei_password: env::var("DOSEI_PASSWORD")
        .unwrap_or(defaults::DOSEI_USER_PASSWORD.to_string()),
      jwt_secret: env::var("DOSEID_JWT_SECRET").unwrap_or_else(|_| {
        let random_id: String = rand::thread_rng()
          .sample_iter(&Alphanumeric)
          .take(36)
          .map(char::from)
          .collect();
        warn!(
          "No JWT_SECRET provided - generated random secret. \
        This may cause problems when Dosei operates on cluster mode. \
        To provide a shared secret set the DOSEID_JWT_SECRET environment variable."
        );
        random_id
      }),
      disable_telemetry: env::var("DOSEID_DISABLE_TELEMETRY")
        .unwrap_or(defaults::DISABLE_TELEMETRY.to_string())
        .parse()
        .unwrap_or(true),
    })
  }

  pub fn address(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }
}

fn config_from_file(path: &Path) -> anyhow::Result<Value> {
  if !path.join("node_modules").exists() {
    let package_managers = [
      ("package-lock.json", "npm"),
      ("yarn.lock", "yarn"),
      ("pnpm-lock.yaml", "pnpm"),
    ];
    for (lock_file, manager) in &package_managers {
      if path.join(lock_file).exists() {
        std::process::Command::new(manager)
          .arg("install")
          .current_dir(path)
          .stdout(Stdio::inherit())
          .stderr(Stdio::inherit())
          .output()
          .context(format!("Failed to run {} install", manager))?;
        break;
      }
    }
  }

  std::process::Command::new("node")
    .arg(".")
    .current_dir(path)
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .output()
    .context("Failed to read doseid config")?;

  let config_path = path.join(".dosei/doseid_config.json");

  let config_data = fs::read_to_string(&config_path)
    .context(format!("Failed to read config file at {:?}", config_path))?;

  let config: Value = serde_json::from_str(&config_data).context("Failed to parse JSON data")?;
  Ok(config)
}

pub(crate) async fn create_default_user(pool: Arc<Pool<Postgres>>, config: &'static Config) {
  let password = bcrypt::hash(&config.dosei_password, bcrypt::DEFAULT_COST).unwrap();
  sqlx::query!(
    "
    WITH inserted AS (
        INSERT INTO account (id, name, type)
        VALUES ($1, $2, 'individual')
        ON CONFLICT (name) DO NOTHING
        RETURNING id
    )
    INSERT INTO \"user\" (id, password)
    SELECT $1, $3
    FROM inserted
    ",
    Uuid::default(),
    "dosei",
    password
  )
  .execute(&*pool)
  .await
  .unwrap();
}
