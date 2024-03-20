use bollard::container::{CreateContainerOptions, StartContainerOptions};
use bollard::models::{HostConfig, PortBinding, PortMap};
use bollard::Docker;
use std::collections::HashMap;
use tracing::error;

pub const CONSOLE_SERVICE_NAME: &str = "dosei_console";

pub(crate) async fn shutdown_app() -> anyhow::Result<()> {
  let docker = Docker::connect_with_socket_defaults().unwrap();
  docker.stop_container(CONSOLE_SERVICE_NAME, None).await?;
  docker.remove_container(CONSOLE_SERVICE_NAME, None).await?;
  Ok(())
}

pub async fn start_app() -> anyhow::Result<()> {
  let port = 3000;
  let exposed_port = format!("{}/tcp", &port);

  // Initialize exposed ports map
  let empty = HashMap::new();
  let mut exposed_ports = HashMap::new();
  exposed_ports.insert(exposed_port.as_str(), empty);

  // Initialize port bindings
  let port_binding = vec![PortBinding {
    host_ip: Some("127.0.0.1".to_string()),
    host_port: Some(port.to_string()),
  }];
  let mut port_map = PortMap::new();
  port_map.insert(format!("{}/tcp", &port), Some(port_binding));

  let host_config = HostConfig {
    port_bindings: Some(port_map),
    ..Default::default()
  };

  let config = bollard::container::Config {
    image: Some("doseiai/app"),
    exposed_ports: Some(exposed_ports),
    host_config: Some(host_config),
    tty: Some(true),
    env: Some(vec!["EXAMPLE=2"]),
    ..Default::default()
  };
  let docker = Docker::connect_with_socket_defaults().unwrap();
  let options = Some(CreateContainerOptions {
    name: CONSOLE_SERVICE_NAME,
    platform: None,
  });
  let container = docker.create_container(options, config).await.unwrap();
  if let Err(e) = docker
    .start_container(&container.id, None::<StartContainerOptions<String>>)
    .await
  {
    error!("Error starting container: {:?}", e)
  }
  Ok(())
}
