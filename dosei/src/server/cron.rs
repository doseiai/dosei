use tokio::time::sleep;
use std::time::Duration;
use dosei_proto::node_info;
use crate::config;
use crate::config::{Config};
use prost::Message;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::error::Error;

async fn update_status(config: Config) -> Result<(), Box<dyn Error>> {
  let node_info = node_info::NodeInfo {
    uuid: config.node_info.uuid.to_string(),
    r#enum: i32::from(node_info::NodeType::Replica),
    address: config.address.to_string(),
    version: config::version()
  };

  let mut buf = Vec::with_capacity(node_info.encoded_len() + 1);
  buf.push(0x04);

  // Serialize the CronJob instance to a buffer
  node_info.encode(&mut buf)?;

  // Connect to a peer
  let primary_node_address = config.get_primary_node_address().to_string();
  print!("{}", primary_node_address);
  let mut stream = TcpStream::connect(primary_node_address).await?;

  // Write the serialized data
  stream.write_all(&buf).await?;
  Ok(())
}

pub fn start_job_manager(config: &Config) {
  let config = config.clone();
  tokio::spawn(async move {
    loop {
      sleep(Duration::from_secs(1)).await;
      let config = config.clone();
      if config.is_replica() {
        update_status(config).await.unwrap();
      }
      // read_minute_jobs().await;
    }
  });
}
