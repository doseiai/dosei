use clap::Parser;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[arg(short, long)]
  connect: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum NodeType {
  PRIMARY,
  REPLICA,
}

#[derive(Debug)]
pub struct NodeInfo {
  pub uuid: Uuid,
  pub node_type: NodeType
}

#[derive(Debug)]
pub struct Config {
  pub node_info: NodeInfo
}

pub fn init() -> Config {
  let args = Args::parse();
  let node_info: NodeInfo = NodeInfo {
    uuid: Uuid::new_v4(),
    node_type: if args.connect.is_some() { NodeType::REPLICA } else { NodeType::PRIMARY },
  };
  Config { node_info }
}
