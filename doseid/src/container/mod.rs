use crate::server::deployment::schema::Deployment;
use anyhow::{anyhow, Context};
use bollard::container::{CreateContainerOptions, InspectContainerOptions, StartContainerOptions};
use bollard::image::BuildImageOptions;
use bollard::models::{HostConfig, PortBinding, PortMap};
use bollard::Docker;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::env::current_dir;
use std::path::Path;
use std::process::Stdio;
use tracing::error;

pub(crate) async fn check_docker_daemon_status() {
  match Docker::connect_with_socket_defaults() {
    Ok(connection) => match connection.ping().await {
      Ok(_) => {}
      Err(e) => {
        error!("Failed to ping Docker: {}", e);
        std::process::exit(1);
      }
    },
    Err(e) => {
      error!("Failed to connect to Docker: {}", e);
      std::process::exit(1);
    }
  };
}

pub(crate) async fn build_docker_image(image_tag: &str, tar: &[u8]) -> anyhow::Result<Vec<String>> {
  let docker = Docker::connect_with_socket_defaults()?;

  let build_image_options = BuildImageOptions {
    dockerfile: "Dockerfile",
    t: image_tag,
    ..Default::default()
  };

  let mut stream = docker.build_image(build_image_options, None, Some(tar.to_owned().into()));
  let mut logs = Vec::new(); // Vector to store logs

  while let Some(build_result) = stream.next().await {
    match build_result {
      Ok(build_info) => {
        if let Some(stream) = build_info.stream {
          logs.push(stream);
        }
      }
      Err(e) => {
        let error = format!("{:?}", e);
        error!("{}", e);
        logs.push(error);
        break;
      }
    }
  }
  Ok(logs)
}

pub(crate) async fn run_deployment(deployment: &Deployment, image_tag: &str) -> anyhow::Result<()> {
  let docker = Docker::connect_with_socket_defaults()?;

  let exposed_port;
  let exposed_ports = if let Some(container_port) = deployment.container_port {
    let mut container_ports = HashMap::new();
    exposed_port = format!("{}/tcp", container_port);
    container_ports.insert(exposed_port.as_str(), HashMap::new());
    Some(container_ports)
  } else {
    None
  };

  let host_config = if let Some(host_port) = deployment.host_port {
    let mut port_map = PortMap::new();
    // TODO: make this cleaner unwrap, move to exposed port check or something
    port_map.insert(
      format!("{}/tcp", &deployment.container_port.unwrap()),
      Some(vec![PortBinding {
        host_ip: Some("127.0.0.1".to_string()),
        host_port: Some(host_port.to_string()),
      }]),
    );
    Some(HostConfig {
      port_bindings: Some(port_map),
      ..Default::default()
    })
  } else {
    None
  };

  let options = Some(CreateContainerOptions {
    name: deployment.id,
    platform: None,
  });

  let config = bollard::container::Config {
    image: Some(image_tag),
    exposed_ports,
    host_config,
    tty: Some(true),
    ..Default::default()
  };
  let container = docker.create_container(options, config).await?;

  docker
    .start_container(&container.id, None::<StartContainerOptions<String>>)
    .await?;
  Ok(())
}

pub(crate) async fn stop_deployment(deployment: &Deployment) -> anyhow::Result<()> {
  let docker = Docker::connect_with_socket_defaults()?;
  docker
    .stop_container(&deployment.id.to_string(), None)
    .await?;
  Ok(())
}

pub(crate) async fn dosei_init(image_tag: &str) -> anyhow::Result<String> {
  let docker = Docker::connect_with_socket_defaults()?;

  let container = docker
    .create_container(
      None::<CreateContainerOptions<String>>,
      bollard::container::Config {
        image: Some(image_tag),
        cmd: Some(vec!["node", "dosei.js"]),
        ..Default::default()
      },
    )
    .await
    .map_err(|_| anyhow!("Couldn't import dosei app"))?;

  docker
    .start_container(&container.id, None::<StartContainerOptions<String>>)
    .await?;
  Ok(container.id)
}

pub(crate) async fn container_working_dir(container_id: &str) -> anyhow::Result<String> {
  let docker = Docker::connect_with_socket_defaults()?;

  let options = Some(InspectContainerOptions { size: false });

  let inspect_response = docker.inspect_container(container_id, options).await?;
  Ok(inspect_response.config.unwrap().working_dir.unwrap())
}

pub(crate) async fn export_dot_dosei(
  container_id: &str,
  working_dir: &str,
  dst_dir: &Path,
) -> anyhow::Result<()> {
  let dot_dosei_export_path = format!("{}/{}", &working_dir, ".dosei");

  std::process::Command::new("docker")
    .arg("cp")
    .arg(format!("{}:{}", &container_id, &dot_dosei_export_path))
    .arg(dst_dir)
    .current_dir(current_dir().unwrap())
    .stderr(Stdio::inherit())
    .output()
    .context("Failed to export dosei app")?;

  // Not working unless running cp before, can't figure out why, seems bug with the GRCP API
  // and it's only solved over the go api.
  // https://github.com/moby/moby/blob/master/pkg/archive/copy.go#L31C1-L50C2
  // let docker = Docker::connect_with_socket_defaults()?;
  //
  // let output_file_path = current_dir().unwrap().join(".dosei/.dosei.tar.gz");
  // if let Some(dosei_dir) = output_file_path.parent() {
  //   fs::create_dir_all(dosei_dir)?;
  // }
  // println!("{}", format!("{}/.dosei", working_dir));
  // let options = Some(DownloadFromContainerOptions {
  //   path: dot_dosei_export_path,
  // });
  //
  // let mut output_file = File::create(&output_file_path).await?;
  // let mut stream = docker.download_from_container(container_id, options);
  // while let Some(chunk) = stream.next().await {
  //   let data = chunk?;
  //   output_file.write_all(&data).await?;
  // }
  Ok(())
}
