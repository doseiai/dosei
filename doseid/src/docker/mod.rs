pub(crate) mod credentials;
pub(crate) mod event;

use bollard::auth::DockerCredentials;
use bollard::image::{BuildImageOptions, PushImageOptions};
use bollard::Docker;

use crate::util::{read_tar_gz_content, write_tar_gz};
use futures_util::StreamExt;
use std::default::Default;
use std::path::Path;
use tokio::fs::remove_file;
use tracing::{error, info};

pub async fn build_image(image_tag: &str, folder_path: &Path) {
  let docker = Docker::connect_with_socket_defaults().unwrap();

  let output_path = "output.tar.gz";
  write_tar_gz(output_path, folder_path).await.unwrap();

  let build_image_options = BuildImageOptions {
    dockerfile: "Dockerfile",
    t: image_tag,
    ..Default::default()
  };

  let tar = read_tar_gz_content(output_path).await;
  let mut stream = docker.build_image(build_image_options, None, Some(tar.into()));
  while let Some(build_result) = stream.next().await {
    match build_result {
      Ok(build_info) => {
        info!("{:?}", build_info);
      }
      Err(e) => {
        error!("{:?}", e);
        break;
      }
    }
  }
  remove_file(output_path).await.unwrap();
}

pub async fn build_image_raw(image_tag: &str, tar: &[u8]) -> anyhow::Result<Vec<String>> {
  let docker = Docker::connect_with_socket_defaults().unwrap();

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

pub async fn push_image(name: &str, tag: &str, docker_credentials: DockerCredentials) {
  let docker = Docker::connect_with_socket_defaults().unwrap();
  let mut stream = docker.push_image(
    name,
    Some(PushImageOptions { tag }),
    Some(docker_credentials),
  );
  while let Some(push_result) = stream.next().await {
    match push_result {
      Ok(output) => info!("{:?}", output),
      Err(e) => {
        error!("Push error: {:?}", e);
        break;
      }
    }
  }
}
