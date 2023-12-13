use crate::config::{Config};
use dosei_proto::ProtoChannel;
use dosei_proto::{cron_job, node_info};
use log::{error, info};
use prost::Message;
use std::sync::Arc;
use once_cell::sync::Lazy;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

static CLUSTER_INFO: Lazy<Arc<Mutex<ClusterInfo>>> = Lazy::new(|| {
  Arc::new(Mutex::new(ClusterInfo { replicas: Vec::new() }))
});

#[derive(Debug, Clone)]
pub struct ClusterInfo {
  pub replicas: Vec<node_info::NodeInfo>,
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
      match buf.get(0) {
        Some(&node_info::NodeInfo::PROTO_ID) => {
          let received_data = match node_info::NodeInfo::decode(buf_slice) {
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
