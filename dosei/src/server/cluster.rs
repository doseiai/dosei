use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt};
use log::{info, error};
use dosei_proto::cron_job;
use dosei_proto::cluster_node;
use prost::Message;


#[derive(Debug)]
enum ProtoType {
  Ping,
  Pong,
  CronJob,
  ClusterNode,
}

pub fn start_main_node() {
  tokio::spawn(async {
    let main_address = format!("{}:{}", "0.0.0.0", "8844".parse::<i32>().unwrap() + 10000);
    let listener = TcpListener::bind(&main_address).await.expect("Failed to bind to address");
    loop {
      let (mut socket, addr) = listener.accept().await.expect("Failed to accept connection");

      info!("New connection from {}", addr); // Log new instance connection

      let mut buf = Vec::new(); // buffer for reading data

      // Read data into buffer
      let n = match socket.read_to_end(&mut buf).await {
        Ok(n) => n,
        Err(_) => return,
      };
      if n == 0 {
        return;
      }
      info!("Bytes read: {}", n);

      let proto_type = identify_proto_type(&buf); // Function to identify the type
      let buf_slice = &buf[1..];

      // Process data based on identified type
      match proto_type {
        ProtoType::CronJob => {
          let received_data = match cron_job::CronJob::decode(buf_slice) {
            Ok(data) => data,
            Err(e) => {
              error!("Failed to decode CronJob: {}", e);
              continue;
            },
          };
          info!("Received CronJob: {:?}", received_data); // Log the received data
        },
        ProtoType::ClusterNode => {
          let received_data = match cluster_node::ClusterNode::decode(buf_slice) {
            Ok(data) => data,
            Err(e) => {
              error!("Failed to decode ClusterNode: {}", e);
              continue;
            },
          };
          info!("Received ClusterNode: {:?}", received_data); // Log the received data
        },
        ProtoType::Ping | ProtoType::Pong => todo!(),
        // Add more cases as needed
      }
    }
  });
  info!("Dosei Node main initialized");
}

fn identify_proto_type(buf: &[u8]) -> ProtoType {
  match buf.get(0) {
    Some(&0x01) => ProtoType::Ping,
    Some(&0x02) => ProtoType::Pong,
    Some(&0x03) => ProtoType::CronJob,
    Some(&0x04) => ProtoType::ClusterNode,
    // Add more cases as needed
    _ => panic!("Unknown protocol type"),
  }
}
