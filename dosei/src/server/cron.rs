use crate::config::Config;
use crate::schema::{CronJob, Job};
use axum::http::StatusCode;
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
use futures_util::stream::StreamExt;
use gcp_auth::AuthenticationManager;
use log::{error, info};
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

pub fn start_job_manager(config: &'static Config, pool: Arc<Pool<Postgres>>) {
  tokio::spawn(async move {
    loop {
      run_jobs(config, Arc::clone(&pool)).await;
      sleep(Duration::from_secs(60)).await;
    }
  });
  tokio::spawn(async move {
    listen_docker_events().await;
  });
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
) -> Result<Json<CronJob>, StatusCode> {
  let cron_job = CronJob {
    id: Uuid::new_v4(),
    schedule: body.schedule,
    entrypoint: body.entrypoint,
    owner_id: body.owner_id,
    deployment_id: body.deployment_id,
    updated_at: Default::default(),
    created_at: Default::default(),
  };
  match sqlx::query_as!(
    CronJob,
    "
    INSERT INTO cron_jobs (id, schedule, entrypoint, owner_id, deployment_id, updated_at, created_at)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
    RETURNING *
    ",
    cron_job.id,
    cron_job.schedule,
    cron_job.entrypoint,
    cron_job.owner_id,
    cron_job.deployment_id,
    cron_job.updated_at,
    cron_job.created_at
  ).fetch_one(&**pool).await
  {
    Ok(recs) => Ok(Json(recs)),
    Err(err) => {
      error!("Error in creating job: {:?}", err);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

pub async fn api_get_cron_jobs(
  pool: Extension<Arc<Pool<Postgres>>>,
) -> Result<Json<Vec<CronJob>>, StatusCode> {
  match get_cron_jobs(pool.0).await {
    Ok(result) => Ok(result),
    Err(err) => {
      error!("Error in reading job: {:?}", err);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

async fn get_cron_jobs(pool: Arc<Pool<Postgres>>) -> Result<Json<Vec<CronJob>>, sqlx::Error> {
  match sqlx::query_as!(CronJob, "SELECT * from cron_jobs")
    .fetch_all(&*pool)
    .await
  {
    Ok(recs) => Ok(Json(recs)),
    Err(err) => Err(err),
  }
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
      Err(e) => error!("Docker streaming failed: {:?}", e),
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
    id: Uuid::new_v4(),
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

async fn run_job(config: &'static Config, cron_job: CronJob) {
  let docker = Docker::connect_with_socket_defaults().unwrap();

  let image_name = "alw3ys/dosei-bot";
  let image_tag = format!(
    "{}/{}:{}",
    &config.container_registry_url, &image_name, &cron_job.deployment_id
  );

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

async fn run_jobs(config: &'static Config, pool: Arc<Pool<Postgres>>) {
  let cron_jobs = get_cron_jobs(pool).await.unwrap();
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
          &job.id, &job.schedule, &job.entrypoint
        );
        run_job(config, job).await;
      }
    }
  }
}
