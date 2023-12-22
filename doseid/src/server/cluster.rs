use crate::config;
use crate::config::Config;
use dosei_proto::ProtoChannel;
use dosei_proto::{cron_job, ping};
use once_cell::sync::Lazy;
use prost::Message;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info};

static CLUSTER_INFO: Lazy<Arc<Mutex<ClusterInfo>>> = Lazy::new(|| {
  Arc::new(Mutex::new(ClusterInfo {
    replicas: Vec::new(),
  }))
});

pub fn start_cluster(config: &'static Config) -> anyhow::Result<()> {
  start_node(config);
  if config.is_replica() {
    tokio::spawn(async move {
      loop {
        sleep(Duration::from_secs(1)).await;
        update_status(config).await.unwrap();
      }
    });
  }
  Ok(())
}

pub fn start_node(config: &'static Config) {
  let cluster_info = Arc::clone(&CLUSTER_INFO);
  let address = config.node_info.address.clone();
  tokio::spawn(async move {
    let listener = TcpListener::bind((address.host, address.port))
      .await
      .unwrap();
    loop {
      let (mut socket, _) = listener.accept().await.unwrap();

      let mut buf = Vec::new(); // buffer for reading data

      // Read data into buffer
      let n = match socket.read_to_end(&mut buf).await {
        Ok(n) => n,
        Err(_) => return,
      };
      if n == 0 {
        return;
      }

      let buf_slice = &buf[1..];

      // Process data based on identified type
      match buf.first() {
        Some(&ping::Ping::PROTO_ID) => {
          let received_data = match ping::Ping::decode(buf_slice) {
            Ok(data) => data,
            Err(e) => {
              error!("Failed to decode ClusterNode: {}", e);
              continue;
            }
          };
          let mut cluster_info = cluster_info.lock().await;
          cluster_info.add_or_update_replica(received_data.clone());
          println!("{:?}", cluster_info);
        }
        Some(&cron_job::CronJob::PROTO_ID) => {
          let received_data = match cron_job::CronJob::decode(buf_slice) {
            Ok(data) => data,
            Err(e) => {
              error!("Failed to decode CronJob: {}", e);
              continue;
            }
          };
          info!("Received CronJob: {:?}", received_data); // Log the received data
        }
        _ => todo!(),
      }
    }
  });
}

async fn update_status(config: &'static Config) -> Result<(), Box<dyn Error>> {
  let node_info = ping::Ping {
    id: config.node_info.id.to_string(),
    node_type: i32::from(config.node_info.node_type),
    address: config.address.to_string(),
    version: config::VERSION.to_string(),
  };

  let mut buf = Vec::with_capacity(node_info.encoded_len() + 1);
  buf.push(ping::Ping::PROTO_ID);

  node_info.encode(&mut buf)?;

  let primary_node_address = config.get_primary_node_address().to_string();
  let mut stream = TcpStream::connect(primary_node_address).await?;

  stream.write_all(&buf).await?;
  Ok(())
}

#[derive(Debug, Clone)]
pub struct ClusterInfo {
  pub replicas: Vec<ping::Ping>,
}

impl ClusterInfo {
  pub fn add_or_update_replica(&mut self, replica: ping::Ping) {
    match self.replicas.iter_mut().find(|r| r.id == replica.id) {
      Some(existing_replica) => {
        *existing_replica = replica;
      }
      None => {
        self.replicas.push(replica);
      }
    }
  }
}
