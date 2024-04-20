use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Deployment {
  pub id: Uuid,
  pub service_id: Uuid,
  pub owner_id: Uuid,
  pub status: DeploymentStatus,
  pub host_port: Option<i16>,
  pub container_port: Option<i16>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
#[sqlx(type_name = "deployment_status", rename_all = "lowercase")]
pub enum DeploymentStatus {
  Pending,
  Error,
  Ready,
  Running,
  Stopped,
}

impl Deployment {
  pub async fn new(
    service_id: Uuid,
    owner_id: Uuid,
    app: &App,
    pool: Arc<Pool<Postgres>>,
  ) -> anyhow::Result<Deployment> {
    let container_port = app.port;
    let host_port = if container_port.is_some() {
      Some(dosei_util::network::find_available_host_port()?)
    } else {
      None
    };

    let deployment = sqlx::query_as!(
    Deployment,
    "
    INSERT INTO deployment (id, service_id, owner_id, host_port, container_port, updated_at, created_at)
    VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, $6, $7)
    RETURNING
    id,
    service_id,
    owner_id,
    host_port,
    container_port,
    status AS \"status!: DeploymentStatus\",
    updated_at,
    created_at
    ",
    Uuid::new_v4(),
    service_id,
    owner_id,
    host_port.map(|p| p as i16),
    container_port.map(|p| p as i16),
    Utc::now(),
    Utc::now()
  )
      .fetch_one(&*pool)
      .await?;
    Ok(deployment)
  }

  pub async fn update_status(
    id: Uuid,
    owner_id: Uuid,
    status: DeploymentStatus,
    pool: Arc<Pool<Postgres>>,
  ) -> anyhow::Result<Deployment> {
    let deployment = sqlx::query_as!(
      Deployment,
      "
      UPDATE deployment
      SET status = $1, updated_at = $2
      WHERE id = $3::uuid AND owner_id::uuid = $4
      RETURNING
      id,
      service_id,
      owner_id,
      host_port,
      container_port,
      status AS \"status!: DeploymentStatus\",
      updated_at,
      created_at
      ",
      status as DeploymentStatus,
      Utc::now(),
      id,
      owner_id,
    )
    .fetch_one(&*pool)
    .await?;
    Ok(deployment)
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct App {
  pub name: Option<String>,
  pub run: Option<String>,
  pub port: Option<u16>,
}

impl App {
  pub fn import_from_dot_dosei(path: &Path) -> anyhow::Result<App> {
    let app_path = path.join(".dosei/app.json");
    let app_data = fs::read_to_string(app_path)?;

    let app = serde_json::from_str::<App>(&app_data)?;
    Ok(app)
  }
}
