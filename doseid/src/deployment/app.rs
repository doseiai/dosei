use bollard::container::{CreateContainerOptions, LogOutput, LogsOptions, StartContainerOptions};
use bollard::Docker;
use dosei_util::Framework;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub async fn import_dosei_app(
  name: &str,
  tag: &str,
  folder_path: &Path,
) -> anyhow::Result<DoseiApp> {
  let app_instance = dosei_util::_find_framework_init(&Framework::Dosei, folder_path)
    .map_err(|_| anyhow::Error::msg("No Dosei initialization found."))?;
  println!("Detected `Dosei` initialization({})", app_instance);

  let docker = Docker::connect_with_socket_defaults().unwrap();

  let container = docker
    .create_container(
      None::<CreateContainerOptions<String>>,
      bollard::container::Config {
        image: Some(format!("{}:{}", name, tag).as_str()),
        cmd: Some(vec![
          "/bin/sh",
          "-c",
          format!("dosei export {} && cat .dosei/app.json", app_instance).as_str(),
        ]),
        ..Default::default()
      },
    )
    .await
    .map_err(|_| anyhow::Error::msg("Couldn't import .dosei/app.json"))?;

  docker
    .start_container(&container.id, None::<StartContainerOptions<String>>)
    .await?;

  let mut logs_stream = docker.logs(
    &container.id,
    Some(LogsOptions::<String> {
      follow: true,
      stdout: true,
      stderr: true,
      timestamps: false,
      ..Default::default()
    }),
  );
  let mut message_accumulator = Vec::new();

  while let Some(log_result) = logs_stream.next().await {
    match log_result {
      Ok(log_output) => match log_output {
        LogOutput::StdOut { message } | LogOutput::StdErr { message } => {
          message_accumulator.extend_from_slice(&message);
        }
        _ => return Err(anyhow::Error::msg("Error deserializing .dosei/app.json")),
      },
      Err(_) => return Err(anyhow::Error::msg("Error deserializing .dosei/app.json")),
    }
  }

  serde_json::from_str::<DoseiApp>(&String::from_utf8_lossy(&message_accumulator))
    .map_err(|_| anyhow::Error::msg("Error deserializing .dosei/app.json"))
    .map(|app| {
      println!("Importing `Dosei` initialization");
      app
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DoseiApp {
  cron_jobs: Vec<CronJob>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CronJob {
  schedule: String,
  entrypoint: String,
  is_async: bool,
}
