use anyhow::Context;
use clap::Parser;
use dosei_proto::ping::NodeType;
use dotenv::dotenv;
use std::fmt::Formatter;
use std::{env, fmt};
use uuid::Uuid;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

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
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
  pub id: Uuid,
  pub node_type: NodeType,
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

#[derive(Debug, Clone)]
pub struct Config {
  pub address: Address,
  pub node_info: NodeInfo,
  pub primary_address: Option<String>,
  pub container_registry_url: String,
  pub telemetry_disabled: bool,
}

impl Config {
  #[allow(dead_code)]
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
    container_registry_url: env::var("CONTAINER_REGISTRY_URL")
      .context("CONTAINER_REGISTRY_URL is required.")?,
    telemetry_disabled: args.disable_telemetry.unwrap_or(false)
      || env::var("DOSEID_TELEMETRY_DISABLED")
        .map(|v| v == "true")
        .unwrap_or(false),
  })
}
