use bollard::auth::DockerCredentials;
use bollard::image::{BuildImageOptions, PushImageOptions};
use bollard::Docker;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use futures_util::StreamExt;
use gcp_auth::AuthenticationManager;
use std::default::Default;
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use tar::{Archive, Builder};
use tokio::fs::remove_file;
use tokio::task;
use tracing::{error, info};

pub async fn gcr_credentials() -> DockerCredentials {
  let authentication_manager = AuthenticationManager::new().await.unwrap();
  let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
  let token = authentication_manager.get_token(scopes).await.unwrap();
  DockerCredentials {
    username: Some("oauth2accesstoken".to_string()),
    password: Some(token.as_str().to_string()),
    ..Default::default()
  }
}

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

async fn write_tar_gz(output_path: &str, folder_path: &Path) -> anyhow::Result<()> {
  let output_path = output_path.to_owned();
  let folder_path = folder_path.to_path_buf();
  task::spawn_blocking(move || {
    let tar_gz = File::create(output_path)?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(enc);

    tar.append_dir_all(".", folder_path)?;

    tar.into_inner()?.finish()?;
    Ok(())
  })
  .await?
}

pub(crate) async fn extract_tar_gz_from_memory(
  combined_data: &[u8],
  target_folder: &Path,
) -> anyhow::Result<()> {
  let combined_data_owned = combined_data.to_owned();
  let target_folder_buf = target_folder.to_path_buf();
  task::spawn_blocking(move || {
    let cursor = Cursor::new(combined_data_owned);
    let decoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(decoder);
    archive.unpack(target_folder_buf)?;
    Ok(())
  })
  .await?
}

pub(crate) async fn read_tar_gz_content(output_path: &str) -> Vec<u8> {
  use tokio::fs::File;
  use tokio::io::AsyncReadExt;

  let mut file = File::open(output_path).await.unwrap();
  let mut contents = Vec::new();
  file.read_to_end(&mut contents).await.unwrap();
  contents
}
