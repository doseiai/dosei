use crate::config;
use crate::config::Config;
use crate::schema::CronJob;
use axum::{Extension, Json};
use bollard::container::CreateContainerOptions;
use bollard::Docker;
use chrono::Utc;
use cron::Schedule;
use dosei_proto::{node_info, ProtoChannel};
use log::info;
use prost::Message;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::sleep;
use uuid::Uuid;

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
  deployment_id: Uuid,
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

async fn run_job(cron_job: CronJob) {
  let docker = Docker::connect_with_socket_defaults().unwrap();

  let options = Some(CreateContainerOptions {
    name: "",
    platform: None,
  });

  let image_name = "us-docker.pkg.dev/serious-sublime-394315/builds/alw3ys/dosei-bot";
  let x = format!("{}:{}", &image_name, &cron_job.deployment_id);
  let config = bollard::container::Config {
    image: Some(x.as_str()),
    cmd: Some(vec!["dosei", "run", &cron_job.entrypoint]),
    ..Default::default()
  };

  let container = docker.create_container(options, config).await.unwrap();
  docker
    .start_container::<&str>(&container.id, None)
    .await
    .unwrap();
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
        // run_job(job).await;
      }
    }
  }
}

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
}
