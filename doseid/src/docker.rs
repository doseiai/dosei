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
use bollard::container::{CreateContainerOptions, LogOutput, LogsOptions, StartContainerOptions};
use serde::{Deserialize, Serialize};
use tar::Builder;
use tokio::fs::remove_file;
use tokio::task;

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

pub async fn run_command(name: &str, tag: &str, app_instance: &str) {
  let docker = Docker::connect_with_socket_defaults().unwrap();

  let image_tag = format!("{}:{}", name, tag);
  let command = format!("dosei export {} && cat .dosei/app.json", app_instance);

  let config = bollard::container::Config {
    image: Some(image_tag.as_str()),
    cmd: Some(vec!["/bin/sh", "-c", command.as_str()]),
    ..Default::default()
  };

  let container = docker
    .create_container(None::<CreateContainerOptions<String>>, config)
    .await
    .unwrap();

  docker.start_container(&container.id, None::<StartContainerOptions<String>>).await.unwrap();

  let logs_options = LogsOptions::<String> {
    follow: true,
    stdout: true,
    stderr: true,
    timestamps: false,
    ..Default::default()
  };
  let mut logs_stream = docker.logs(&container.id, Some(logs_options));
  let mut message_accumulator = Vec::new(); // to accumulate message bytes

  while let Some(log_result) = logs_stream.next().await {
    match log_result {
      Ok(log_output) => match log_output {
        LogOutput::StdOut { message } | LogOutput::StdErr { message } => {
          message_accumulator.extend_from_slice(&message);
        }
        _ => eprintln!("Received other type of log"),
      },
      Err(e) => eprintln!("Error fetching logs: {}", e),
    }
  }
  // Attempt to deserialize the complete JSON string.
  let complete_message = String::from_utf8_lossy(&message_accumulator);
  let dosei_app: Result<DoseiApp, _> = serde_json::from_str(&complete_message);

  match dosei_app {
    Ok(app) => println!("{:?}", app),
    Err(e) => eprintln!("Error deserializing message: {}", e),
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct CronJob {
  schedule: String,
  entrypoint: String,
  is_async: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct DoseiApp {
  cron_jobs: Vec<CronJob>,
}

pub async fn push_image(name: &str, tag: &str) {
  let docker = Docker::connect_with_socket_defaults().unwrap();
  let mut stream = docker.push_image(
    name,
    Some(PushImageOptions { tag }),
    Some(gcr_credentials().await),
  );
  while let Some(push_result) = stream.next().await {
    match push_result {
      Ok(output) => println!("{:?}", output),
      Err(e) => {
        eprintln!("Push error: {:?}", e);
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

async fn read_tar_gz_content(output_path: &str) -> Vec<u8> {
  use tokio::fs::File;
  use tokio::io::AsyncReadExt;

  let mut file = File::open(output_path).await.unwrap();
  let mut contents = Vec::new();
  file.read_to_end(&mut contents).await.unwrap();
  contents
}
