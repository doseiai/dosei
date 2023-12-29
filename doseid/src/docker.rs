use bollard::auth::DockerCredentials;
use bollard::image::{BuildImageOptions, PushImageOptions};
use bollard::Docker;
use flate2::write::GzEncoder;
use flate2::Compression;
use futures_util::StreamExt;
use gcp_auth::AuthenticationManager;
use std::default::Default;
use std::fs::File;
use std::hash::Hasher;
use std::io::prelude::*;
use std::path::Path;
use tar::Builder;
use tokio::fs::remove_file;

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

// TODO: This sucks, is blocking, refactor.
pub async fn build_image(name: &str, tag: &str, folder_path: &Path) {
  let docker = Docker::connect_with_socket_defaults().unwrap();
  let output_path = "output.tar.gz";
  let tar_gz = File::create(output_path).unwrap();
  let enc = GzEncoder::new(tar_gz, Compression::default());
  let mut tar = Builder::new(enc);
  tar.append_dir_all(".", folder_path).unwrap();
  tar.into_inner().unwrap().finish().unwrap();

  let build_image_options = BuildImageOptions {
    dockerfile: "Dockerfile",
    t: &format!("{}:{}", name, tag),
    ..Default::default()
  };

  let mut file = File::open(output_path).unwrap();
  let mut contents = Vec::new();
  file.read_to_end(&mut contents).unwrap();
  let mut stream = docker.build_image(build_image_options, None, Some(contents.into()));
  while let Some(build_result) = stream.next().await {
    match build_result {
      Ok(build_info) => {
        println!("Build info: {:?}", build_info);
      }
      Err(e) => {
        eprintln!("Build error: {:?}", e);
        break;
      }
    }
  }
  remove_file(output_path).await.unwrap();
}

async fn push_image(name: &str, tag: &str) {
  let docker = Docker::connect_with_socket_defaults().unwrap();
  let _ = docker.push_image(
    name,
    Some(PushImageOptions { tag }),
    Some(gcr_credentials().await),
  );
}
