use crate::server::integration::github::GithubIntegration;
use anyhow::Context;
use clap::Parser;
use dosei_proto::ping::NodeType;
use dotenv::dotenv;
use home::home_dir;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::fs::{create_dir_all, File};
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{env, fmt, fs, write};
use uuid::Uuid;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

const CONFIG_TEMPLATE: &str = include_str!("../resources/doseid.toml");
const DEFAULT_CONFIG_PATH: &str = ".dosei/doseid/data/doseid.toml";

pub const DEPLOYMENT_LOG_PATH: &str = ".dosei/doseid/data/deployments/logs";
const TELEMETRY_ID_PATH: &str = ".dosei/doseid/data/id";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, disable_help_flag = true)]
struct Args {
  #[arg(long, default_value = "127.0.0.1", help = "The host address to use.")]
  host: String,
  #[arg(short, long, default_value = "8844", help = "The port number to use.")]
  port: u16,
  #[arg(short, long, help = "Primary cluster node's address to connect to.")]
  connect: Option<String>,
  #[arg(long, hide = true, action = clap::ArgAction::SetTrue)]
  disable_telemetry: Option<bool>,
  #[arg(long, action = clap::ArgAction::Help, help = "Print help")]
  help: Option<bool>,
  #[arg(long, help = "Path to doseid TOML config file")]
  config_path: Option<String>,
}

pub struct Config {
  pub address: Address,
  pub node_info: NodeInfo,
  pub primary_address: Option<String>,
  pub database_url: String,
  pub jwt_secret: String,
  pub container_registry_url: String,
  pub telemetry: Telemetry,
  pub github_integration: Option<GithubIntegration>,
}

impl Config {
  /// Creates a new Config instance.
  ///
  /// Configuration settings are determined based on a defined hierarchy of importance:
  /// 1. Command-line arguments
  /// 2. Environment variables
  /// 3. Configuration file
  ///
  pub fn new() -> anyhow::Result<Config> {
    dotenv().ok();
    let mut args = Args::parse();
    if env::var("RUST_LOG").is_err() {
      env::set_var("RUST_LOG", "info");
    }

    let subscriber = tracing_subscriber::fmt()
      .with_line_number(true)
      .with_target(true)
      .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Save default config
    let mut dst_path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    dst_path.push(DEFAULT_CONFIG_PATH);
    if let Some(parent_dir) = dst_path.parent() {
      if create_dir_all(parent_dir).is_ok() && !dst_path.exists() {
        fs::write(&dst_path, CONFIG_TEMPLATE)?;
      }
    }
    if args.config_path.is_none() {
      args.config_path = Some(dst_path.to_str().unwrap().to_string());
    }

    let mut github_integration = None;
    // So ugly, wtf, but right now it works
    if cfg!(test) {
      github_integration = Some(GithubIntegration::new()?);
    } else if let Ok(toml_config) = TOMLConfig::new(args.config_path) {
      if toml_config.github.unstable.enabled {
        github_integration = Some(GithubIntegration::new()?);
      }
    };

    Ok(Config {
      address: Address {
        host: args.host.clone(),
        port: args.port,
      },
      node_info: NodeInfo {
        id: Uuid::new_v4(),
        node_type: if args.connect.is_some() {
          NodeType::Replica
        } else {
          NodeType::Primary
        },
        address: Address {
          host: args.host,
          port: args.port + 10000,
        },
      },
      primary_address: args.connect,
      database_url: env::var("DATABASE_URL").context("DATABASE_URL is required.")?,
      jwt_secret: env::var("JWT_SECRET").context("JWT_SECRET is required.")?,
      container_registry_url: env::var("CONTAINER_REGISTRY_URL")
        .context("CONTAINER_REGISTRY_URL is required.")?,
      telemetry: Telemetry::new()
        .enabled(
          args.disable_telemetry.unwrap_or(false)
            || env::var("DOSEID_TELEMETRY_DISABLED")
              .map(|v| v == "true")
              .unwrap_or(false),
        )
        .build(),
      github_integration,
    })
  }

  pub fn is_primary(&self) -> bool {
    self.node_info.node_type == NodeType::Primary
  }
  pub fn is_replica(&self) -> bool {
    self.node_info.node_type == NodeType::Replica
  }
  pub fn get_primary_node_address(&self) -> Address {
    if let Some(primary_addr) = self.get_primary_address() {
      Address {
        host: primary_addr.host,
        port: primary_addr.port + 10000,
      }
    } else {
      self.node_info.address.clone()
    }
  }

  fn get_primary_address(&self) -> Option<Address> {
    self.primary_address.as_ref().and_then(|addr| {
      let parts: Vec<&str> = addr.split(':').collect();
      if parts.len() == 2 {
        let host = parts[0].to_string();
        if let Ok(port) = parts[1].parse::<u16>() {
          return Some(Address { host, port });
        }
      }
      None
    })
  }
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
  pub id: Uuid,
  pub node_type: NodeType,
  pub address: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
  pub host: String,
  pub port: u16,
}

impl fmt::Display for Address {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}:{}", self.host, self.port)
  }
}

pub struct Telemetry {
  pub client: Option<PostHogClient>,
}

impl Telemetry {
  pub fn is_disabled(&self) -> bool {
    self.client.is_none()
  }

  fn new() -> Telemetry {
    Telemetry { client: None }
  }
  fn enabled(mut self, value: bool) -> Telemetry {
    self.client = match value {
      true => None,
      false => {
        let mut path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        path.push(TELEMETRY_ID_PATH);
        let dir = path.parent().unwrap();
        if !dir.exists() {
          let _ = fs::create_dir_all(dir);
        }
        let uuid = match File::open(&path) {
          Ok(mut file) => {
            let mut content = String::new();
            match file.read_to_string(&mut content) {
              Ok(_) => content,
              Err(_) => Uuid::new_v4().to_string(),
            }
          }
          Err(_) => Uuid::new_v4().to_string(),
        };

        let _ = File::create(&path).and_then(|mut file| write!(file, "{}", uuid));

        Some(PostHogClient {
          id: uuid,
          api_endpoint: "https://app.posthog.com/capture".to_string(),
          project_api_key: "phc_oMPDQ6wwINgWo7tdfIw8btoksBWkrn5Pq0DgPjBFw6E".to_string(),
        })
      }
    };
    self
  }
  fn build(self) -> Telemetry {
    self
  }
}

pub struct PostHogClient {
  id: String,
  api_endpoint: String,
  project_api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CaptureEvent {
  api_key: String,
  event: String,
  distinct_id: String,
  properties: HashMap<String, serde_json::Value>,
}

impl PostHogClient {
  #[allow(dead_code)]
  pub async fn capture(&self) {
    todo!();
  }
  pub async fn identify(&self) {
    let mut set: HashMap<String, String> = HashMap::new();
    set.insert("version".to_string(), VERSION.to_string());
    let mut properties: HashMap<String, serde_json::Value> = HashMap::new();
    properties.insert("$set".to_string(), json!(set));
    let _ = reqwest::Client::new()
      .post(&self.api_endpoint)
      .header(CONTENT_TYPE, "application/json")
      .json(&CaptureEvent {
        api_key: self.project_api_key.clone(),
        event: "$identify".to_string(),
        distinct_id: self.id.to_string(),
        properties,
      })
      .send()
      .await;
  }
}

#[derive(Deserialize)]
pub struct TOMLConfig {
  github: GithubTOML,
}

#[derive(Deserialize)]
pub struct GithubTOML {
  unstable: GithubUnstableTOML,
}

#[derive(Deserialize)]
pub struct GithubUnstableTOML {
  enabled: bool,
}

impl TOMLConfig {
  pub fn new(config: Option<String>) -> anyhow::Result<TOMLConfig> {
    let filename = match config {
      None => return Err(anyhow::Error::msg("Config file not provided")),
      Some(filename) => filename,
    };
    let contents =
      fs::read_to_string(filename).map_err(|_| anyhow::Error::msg("Could not read config file"))?;
    let data: TOMLConfig =
      toml::from_str(&contents).map_err(|_| anyhow::Error::msg("Could not parse config file"))?;
    Ok(data)
  }
}
