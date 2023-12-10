use tokio::time::sleep;
use std::time::Duration;
use dosei_proto::{node_info, ProtoChannel};
use crate::config;
use crate::config::{Config};
use prost::Message;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use std::error::Error;
use bollard::container::CreateContainerOptions;
use bollard::Docker;
use crate::schema::CronJob;

async fn update_status(config: Config) -> Result<(), Box<dyn Error>> {
  let node_info = node_info::NodeInfo {
    uuid: config.node_info.uuid.to_string(),
    r#enum: i32::from(config.node_info.node_type),
    address: config.address.to_string(),
    version: config::version()
  };

  let mut buf = Vec::with_capacity(node_info.encoded_len() + 1);
  buf.push(node_info::NodeInfo::PROTO_ID);

  // Serialize the CronJob instance to a buffer
  node_info.encode(&mut buf)?;

  // Connect to a peer
  let primary_node_address = config.get_primary_node_address().to_string();
  let mut stream = TcpStream::connect(primary_node_address).await?;

  // Write the serialized data
  stream.write_all(&buf).await?;
  Ok(())
}

async fn run_job(cron_job: CronJob) {
  let docker = Docker::connect_with_socket_defaults().unwrap();

  let options = Some(CreateContainerOptions{
    name: "",
    platform: None,
  });

  let image_name = "us-docker.pkg.dev/serious-sublime-394315/builds/alw3ys/dosei-bot";
  let x = format!("{}:{}", &image_name, &cron_job.deployment_id);
  let config = bollard::container::Config {
    image: Some(x.as_str()),
    cmd: Some(vec!["dosei", "run", &cron_job.entrypoint]),
    ..Default::default()
  };

  let container = docker.create_container(options, config).await.unwrap();
  docker.start_container::<&str>(&container.id, None).await.unwrap();
}

async fn run_jobs() {
  loop {
    sleep(Duration::from_secs(60)).await;
  }
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
    }
  });
  tokio::spawn(run_jobs());
}
