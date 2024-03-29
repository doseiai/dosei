use anyhow::anyhow;
use bollard::container::{CreateContainerOptions, LogOutput, LogsOptions, StartContainerOptions};
use bollard::Docker;
use dosei_util::Framework;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::info;

pub async fn import_dosei_app(image_tag: &str, folder_path: &Path) -> anyhow::Result<DoseiApp> {
  let app_instance = dosei_util::_find_framework_init(&Framework::Dosei, folder_path)
    .map_err(|_| anyhow!("No Dosei initialization found."))?;
  info!("Detected `Dosei` initialization({})", app_instance);

  let docker = Docker::connect_with_socket_defaults().unwrap();

  let container = docker
    .create_container(
      None::<CreateContainerOptions<String>>,
      bollard::container::Config {
        image: Some(image_tag),
        cmd: Some(vec![
          "/bin/sh",
          "-c",
          r#"python -c "from dosei_sdk import main; main.export()" && cat .dosei/app.json"#,
        ]),
        ..Default::default()
      },
    )
    .await
    .map_err(|_| anyhow!("Couldn't import .dosei/app.json"))?;

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
        _ => return Err(anyhow!("Error deserializing .dosei/app.json")),
      },
      Err(_) => return Err(anyhow!("Error deserializing .dosei/app.json")),
    }
  }

  serde_json::from_str::<DoseiApp>(&String::from_utf8_lossy(&message_accumulator))
    .map_err(|_| anyhow!("Error deserializing .dosei/app.json"))
    .map(|app| {
      info!("Importing `Dosei` initialization");
      app
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DoseiApp {
  pub name: String,
  pub run: String,
  pub port: u16,
  pub cron_jobs: Vec<CronJob>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CronJob {
  pub schedule: String,
  pub entrypoint: String,
  pubis_async: bool,
}

// "js" | "mjs" | "cjs" | ".ts" | "tsx" => {
// let node_command = r#"
//         (async () => {
//           const { export_config } = await import('@dosei/dosei');
//           await export_config();
//         })();"#;
// if let Err(err) = Command::new("node")
// .arg("-e")
// .arg(node_command)
// .env("NODE_PATH", ".")
// .stdout(Stdio::inherit())
// .stderr(Stdio::inherit())
// .output()
// {
// eprintln!("{:?}", err);
// };
// }
