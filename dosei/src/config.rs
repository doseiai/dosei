use clap::Parser;
use uuid::Uuid;
use dosei_proto::node_info;
use tokio::sync::Mutex;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, disable_help_flag = true)]
struct Args {
  #[arg(short, long, default_value = "127.0.0.1")]
  host: String,
  #[arg(short, long, default_value = "8844")]
  port: u16,
  #[arg(short, long)]
  connect: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeType {
  PRIMARY,
  REPLICA,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
  pub uuid: Uuid,
  pub node_type: NodeType,
  pub address: Address
}

#[derive(Debug, Clone)]
pub struct Address {
  pub host: String,
  pub port: u16,
}

impl Address {
  pub fn to_string(&self) -> String {
    format!("{}:{}", self.host, self.port)
  }
}

#[derive(Debug, Clone)]
pub struct ClusterInfo {
  pub replicas: Vec<node_info::NodeInfo>
}

impl ClusterInfo {
  pub fn add_or_update_replica(&mut self, replica: node_info::NodeInfo) {
    match self.replicas.iter_mut().find(|r| r.uuid == replica.uuid) {
      Some(existing_replica) => {
        *existing_replica = replica;
      }
      None => {
        self.replicas.push(replica);
      }
    }
  }
}

#[derive(Debug, Clone)]
pub struct Config {
  pub address: Address,
  pub cluster_info:  Arc<Mutex<ClusterInfo>>,
  pub node_info: NodeInfo,
  pub primary_address: Option<String>
}


impl Config {
  #[allow(dead_code)]
  pub fn is_primary(&self) -> bool {
    self.node_info.node_type == NodeType::PRIMARY
  }
  pub fn is_replica(&self) -> bool {
    self.node_info.node_type == NodeType::REPLICA
  }
  pub fn get_primary_node_address(&self) -> Address {
    if let Some(primary_addr) = self.get_primary_address() {
      return Address {
        host: primary_addr.host,
        port: primary_addr.port + 10000,
      };
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

pub fn version() -> String {
  let version = env!("CARGO_PKG_VERSION");
  return version.parse().unwrap();
}

pub fn init() -> Config {
  let args = Args::parse();
  Config {
    address: Address {
      host: args.host.clone(),
      port: args.port
    },
    cluster_info: Arc::new(Mutex::new(ClusterInfo { replicas: Vec::new() })),
    node_info: NodeInfo {
      uuid: Uuid::new_v4(),
      node_type: if args.connect.is_some() { NodeType::REPLICA } else { NodeType::PRIMARY },
      address: Address {
        host: args.host,
        port: args.port + 10000
      },
    },
    primary_address: args.connect,
  }
}
