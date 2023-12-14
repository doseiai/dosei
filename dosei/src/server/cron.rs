use crate::config;
use crate::config::Config;
use crate::schema::{CronJob, Job};
use axum::{Extension, Json};
use bollard::auth::DockerCredentials;
use bollard::container::{
  CreateContainerOptions, InspectContainerOptions, LogOutput, LogsOptions, StartContainerOptions,
};
use bollard::image::{CreateImageOptions, ListImagesOptions};
use bollard::models::{ContainerInspectResponse, EventMessage, EventMessageTypeEnum};
use bollard::system::EventsOptions;
use bollard::Docker;
use chrono::Utc;
use cron::Schedule;
use dosei_proto::{node_info, ProtoChannel};
use futures_util::stream::StreamExt;
use gcp_auth::AuthenticationManager;
use log::{error, info};
use prost::Message;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::sleep;
use uuid::Uuid;

pub fn start_job_manager(config: &'static Config, pool: Arc<Pool<Postgres>>) {
  tokio::spawn(async move {
    loop {
      sleep(Duration::from_secs(1)).await;
      if config.is_replica() {
        update_status(config).await.unwrap();
      }
    }
  });
  tokio::spawn(async move {
    loop {
      run_jobs(Arc::clone(&pool)).await;
      sleep(Duration::from_secs(60)).await;
    }
  });
  tokio::spawn(async move {
    listen_docker_events().await;
  });
}

async fn update_status(config: &'static Config) -> Result<(), Box<dyn Error>> {
  let node_info = node_info::NodeInfo {
    uuid: config.node_info.uuid.to_string(),
    r#enum: i32::from(config.node_info.node_type),
    address: config.address.to_string(),
    version: config::VERSION.to_string(),
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

#[derive(Deserialize)]
pub struct CreateJobBody {
  schedule: String,
  entrypoint: String,
  deployment_id: String,
  owner_id: Uuid,
}

pub async fn api_create_job(
  pool: Extension<Arc<Pool<Postgres>>>,
  Json(body): Json<CreateJobBody>,
) -> Json<CronJob> {
  let cron_job = CronJob {
    uuid: Uuid::new_v4(),
    schedule: body.schedule,
    entrypoint: body.entrypoint,
    owner_id: body.owner_id,
    deployment_id: body.deployment_id,
    updated_at: Default::default(),
    created_at: Default::default(),
  };
  let rec = sqlx::query_as!(
    CronJob,
    r#"
    INSERT INTO cron_jobs (uuid, schedule, entrypoint, owner_id, deployment_id, updated_at, created_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    RETURNING *
    "#,
    cron_job.uuid,
    cron_job.schedule,
    cron_job.entrypoint,
    cron_job.owner_id,
    cron_job.deployment_id,
    cron_job.updated_at,
    cron_job.created_at
  ).fetch_one(&**pool).await.unwrap();
  Json(rec)
}

pub async fn api_get_cron_jobs(pool: Extension<Arc<Pool<Postgres>>>) -> Json<Vec<CronJob>> {
  get_cron_jobs(pool.0).await
}

async fn get_cron_jobs(pool: Arc<Pool<Postgres>>) -> Json<Vec<CronJob>> {
  let recs = sqlx::query_as!(CronJob, "SELECT * from cron_jobs")
    .fetch_all(&*pool)
    .await
    .unwrap();
  Json(recs)
}

async fn listen_docker_events() {
  let docker = Docker::connect_with_socket_defaults().unwrap();

  let mut filters = HashMap::new();
  filters.insert("type", vec!["container"]);
  // filters.insert("event", vec!["start", "stop"]); // Listen for start and stop events

  let options = EventsOptions {
    filters,
    ..Default::default()
  };

  let mut stream = docker.events(Some(options));

  while let Some(event_result) = stream.next().await {
    match event_result {
      Ok(event) => {
        let event: EventMessage = event;
        match event.typ {
          Some(EventMessageTypeEnum::CONTAINER) => match event.action.clone().unwrap().as_str() {
            "create" => {
              println!("create");
            }
            "start" => {
              println!("start");
            }
            "die" => {
              error!("die");
              let actor = event.actor.unwrap();
              let job = new_job_from_event(&actor.id.unwrap()).await;
              println!("{:?}", job);
            }
            _ => {}
          },
          Some(EventMessageTypeEnum::BUILDER) => todo!("handle builder events"),
          _ => {}
        }
      }
      Err(e) => error!("{:?}", e),
    }
  }
}

async fn new_job_from_event(container_id: &str) -> Job {
  let docker = Docker::connect_with_socket_defaults().unwrap();
  let container_details: ContainerInspectResponse = docker
    .inspect_container(container_id, None::<InspectContainerOptions>)
    .await
    .unwrap();
  let container_state = container_details.state.unwrap();
  let exit_code = container_state.exit_code.unwrap();
  let logs = container_logs(container_id).await.unwrap();
  Job {
    uuid: Uuid::new_v4(),
    cron_job_id: Uuid::new_v4(),
    exit_code: exit_code as u8,
    logs,
    entrypoint: "".to_string(),
    owner_id: Default::default(),
    updated_at: Default::default(),
    created_at: Default::default(),
  }
}

async fn container_logs(container_id: &str) -> Result<Vec<String>, bollard::errors::Error> {
  let docker = Docker::connect_with_socket_defaults().unwrap();
  let logs_options = Some(LogsOptions::<String> {
    follow: false,
    stdout: true,
    stderr: true,
    timestamps: true,
    ..Default::default()
  });

  let mut log_stream = docker.logs(container_id, logs_options);
  let mut log_lines = Vec::new();

  while let Some(log_result) = log_stream.next().await {
    match log_result {
      Ok(log_output) => match log_output {
        LogOutput::StdOut { message } | LogOutput::StdErr { message } => {
          let log_str = String::from_utf8_lossy(&message);
          log_lines.push(log_str.to_string());
        }
        // Add other LogOutput variants handling here if needed
        _ => {}
      },
      Err(e) => {
        eprintln!("Error fetching logs: {}", e);
        return Err(e);
      }
    }
  }
  Ok(log_lines)
}

async fn run_job(cron_job: CronJob) {
  let docker = Docker::connect_with_socket_defaults().unwrap();

  let image_name = "us-docker.pkg.dev/serious-sublime-394315/builds/alw3ys/dosei-bot";
  let image_tag = format!("{}:{}", &image_name, &cron_job.deployment_id);

  // Check if image exists locally
  let mut filters = HashMap::new();
  filters.insert("reference".to_string(), vec![image_tag.to_string()]);
  match docker
    .list_images(Some(ListImagesOptions::<String> {
      all: true,
      filters,
      ..Default::default()
    }))
    .await
  {
    Ok(images) => {
      if images.is_empty() {
        let options = Some(CreateImageOptions {
          from_image: image_name,
          tag: &cron_job.deployment_id,
          ..Default::default()
        });
        let authentication_manager = AuthenticationManager::new().await.unwrap();
        let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
        let token = authentication_manager.get_token(scopes).await.unwrap();
        let credentials = DockerCredentials {
          username: Some("oauth2accesstoken".to_string()),
          password: Some(token.as_str().to_string()),
          ..Default::default()
        };
        let mut stream = docker.create_image(options, None, Some(credentials));
        while let Some(result) = stream.next().await {
          if let Err(e) = result {
            error!("Error occurred while downloading image: {}", e);
            break;
          }
        }
      }
    }
    Err(err) => panic!("{}", err),
  }

  let config = bollard::container::Config {
    image: Some(image_tag.as_str()),
    cmd: Some(vec!["dosei", "run", &cron_job.entrypoint]),
    ..Default::default()
  };

  let container = docker
    .create_container(None::<CreateContainerOptions<String>>, config)
    .await
    .unwrap();

  match docker
    .start_container(&container.id, None::<StartContainerOptions<String>>)
    .await
  {
    Ok(_) => {}
    Err(e) => error!("Error starting container: {:?}", e),
  }
}

async fn run_jobs(pool: Arc<Pool<Postgres>>) {
  let cron_jobs = get_cron_jobs(pool).await;
  let now = Utc::now();
  for job in cron_jobs.0 {
    let job_schedule = format!("0 {} *", &job.schedule);
    let schedule = Schedule::from_str(&job_schedule).unwrap();
    // Get the next scheduled time for the job
    if let Some(next) = schedule.upcoming(Utc).next() {
      let time_difference = next.timestamp() - now.timestamp();

      // Check if the next scheduled time is within the next 60 seconds and in the future
      if (0..60).contains(&time_difference) {
        info!(
          "Job: {} to run {}; {}",
          &job.uuid, &job.schedule, &job.entrypoint
        );
        run_job(job).await;
      }
    }
  }
}
