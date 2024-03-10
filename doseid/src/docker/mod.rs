pub(crate) mod credentials;
pub(crate) mod event;

use bollard::auth::DockerCredentials;
use bollard::image::{BuildImageOptions, PushImageOptions};
use bollard::Docker;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::util::{read_tar_gz_content, write_tar_gz};
use futures_util::StreamExt;
use std::default::Default;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use tar::{Archive, Builder};
use tokio::fs::remove_file;
use tokio::task;
use tracing::{error, info};

pub async fn build_image(name: &str, tag: &str, folder_path: &Path) {
  let docker = Docker::connect_with_socket_defaults().unwrap();

  let output_path = "output.tar.gz";
  write_tar_gz(output_path, folder_path).await.unwrap();

  let build_image_options = BuildImageOptions {
    dockerfile: "Dockerfile",
    t: &format!("{}:{}", name, tag),
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

pub async fn build_image_raw(image_tag: &str, tar: &[u8]) {
  let docker = Docker::connect_with_socket_defaults().unwrap();

  let build_image_options = BuildImageOptions {
    dockerfile: "Dockerfile",
    t: image_tag,
    ..Default::default()
  };

  let mut stream = docker.build_image(build_image_options, None, Some(tar.to_owned().into()));
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
