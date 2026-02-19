use anyhow::anyhow;
use bollard::container::{CreateContainerOptions, StartContainerOptions};
use bollard::image::BuildImageOptions;
use bollard::models::{HostConfig, PortBinding, PortMap};
use bollard::Docker;
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::net::TcpListener;
use tracing::log::warn;
use tracing::{debug, error, info};
use utoipa::ToSchema;
use uuid::Uuid;

pub mod route;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Deployment {
  pub id: Uuid,
  pub service_id: Uuid,
  pub owner_id: Uuid,
  pub host_port: Option<i16>,
  pub container_port: Option<i16>,
  pub last_accessed_at: Option<DateTime<Utc>>,
  pub updated_at: DateTime<Utc>,
  pub created_at: DateTime<Utc>,
  pub node_id: Option<Uuid>,
}

impl Deployment {
  pub async fn new(
    service_id: Uuid,
    owner_id: Uuid,
    container_port: Option<i16>,
    host_port: Option<i16>,
    node_id: Option<Uuid>,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Self> {
    let host_port = match (container_port, host_port) {
      (Some(_), Some(port)) => Some(port),
      (Some(_), None) => Some(Self::find_available_host_port()?),
      (None, _) => None,
    };
    let deployment = sqlx::query_as!(
      Deployment,
      "INSERT INTO
        deployment (
          id,
          service_id,
          owner_id,
          host_port,
          container_port,
          node_id,
          updated_at,
          created_at
        )
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
       RETURNING *
      ",
      Uuid::new_v4(),
      service_id,
      owner_id,
      host_port,
      container_port,
      node_id,
      Utc::now(),
      Utc::now(),
    )
    .fetch_one(pg_pool)
    .await?;
    info!(
      "Created deployment: {} with host port {:?}",
      deployment.id, deployment.host_port
    );
    Ok(deployment)
  }

  pub async fn build(&self, tar: &[u8]) -> anyhow::Result<Vec<String>> {
    let docker = Docker::connect_with_socket_defaults()?;

    let build_image_options = BuildImageOptions {
      dockerfile: "Dockerfile",
      t: &self.image_tag(),
      ..Default::default()
    };

    let mut stream = docker.build_image(build_image_options, None, Some(tar.to_owned().into()));
    let mut logs = Vec::new(); // Vector to store logs

    while let Some(build_result) = stream.next().await {
      match build_result {
        Ok(build_info) => {
          if let Some(stream) = build_info.stream {
            logs.push(stream);
          }
        }
        Err(e) => {
          let error = format!("{:?}", e);
          error!("{}", e);
          logs.push(error);
          break;
        }
      }
    }
    Ok(logs)
  }

  pub(crate) async fn start(
    &self,
    image_tag: Option<String>,
    env: Option<&HashMap<String, String>>,
  ) -> anyhow::Result<()> {
    let docker = Docker::connect_with_socket_defaults()?;

    let exposed_port;
    let exposed_ports = if let Some(container_port) = self.container_port {
      let mut container_ports = HashMap::new();
      exposed_port = format!("{}/tcp", container_port);
      container_ports.insert(exposed_port, HashMap::new());
      Some(container_ports)
    } else {
      None
    };

    let host_config = if let Some(host_port) = self.host_port {
      let mut port_map = PortMap::new();
      port_map.insert(
        format!("{}/tcp", &self.container_port.unwrap()),
        Some(vec![PortBinding {
          host_ip: Some("127.0.0.1".to_string()),
          host_port: Some(host_port.to_string()),
        }]),
      );
      Some(HostConfig {
        port_bindings: Some(port_map),
        ..Default::default()
      })
    } else {
      None
    };

    let env_vars: Option<Vec<String>> = env.map(|vars| {
      vars.iter().map(|(k, v)| format!("{}={}", k, v)).collect()
    });

    let options = Some(CreateContainerOptions {
      name: self.id,
      platform: None,
    });

    let image_tag = image_tag.unwrap_or(self.image_tag());
    let config = bollard::container::Config {
      image: Some(image_tag),
      exposed_ports,
      host_config,
      env: env_vars,
      tty: Some(true),
      ..Default::default()
    };
    let container = docker.create_container(options, config).await?;

    docker
      .start_container(&container.id, None::<StartContainerOptions<String>>)
      .await?;
    Ok(())
  }

  pub async fn stop(&self) -> anyhow::Result<()> {
    let docker = Docker::connect_with_socket_defaults()?;
    match docker.stop_container(&self.id.to_string(), None).await {
      Ok(_) => Ok(()),
      Err(bollard::errors::Error::DockerResponseServerError { status_code: 404, .. }) => Ok(()),
      Err(bollard::errors::Error::DockerResponseServerError { status_code: 304, .. }) => Ok(()),
      Err(e) => Err(e.into()),
    }
  }

  pub async fn remove(&self) -> anyhow::Result<()> {
    let docker = Docker::connect_with_socket_defaults()?;
    match docker.remove_container(&self.id.to_string(), None).await {
      Ok(_) => Ok(()),
      Err(bollard::errors::Error::DockerResponseServerError { status_code: 404, .. }) => Ok(()),
      Err(e) => Err(e.into()),
    }
  }

  pub async fn delete(&self, pg_pool: &Pool<Postgres>) -> anyhow::Result<()> {
    sqlx::query("DELETE FROM deployment WHERE id = $1")
      .bind(self.id)
      .execute(pg_pool)
      .await?;
    Ok(())
  }

  pub async fn get_by_service_id(
    service_id: Uuid,
    pg_pool: &Pool<Postgres>,
  ) -> anyhow::Result<Vec<Self>> {
    Ok(
      sqlx::query_as!(
        Self,
        "SELECT * FROM deployment WHERE service_id = $1",
        service_id
      )
      .fetch_all(pg_pool)
      .await?,
    )
  }

  pub async fn find_via_host(host: &str, pg_pool: &Pool<Postgres>) -> anyhow::Result<Option<Self>> {
    Ok(
      sqlx::query_as!(
        Self,
        "
         SELECT deployment.* FROM deployment
         JOIN ingress ON deployment.service_id = ingress.service_id
         WHERE ingress.host = $1
         ORDER BY deployment.created_at DESC
         LIMIT 1
        ",
        host
      )
      .fetch_optional(pg_pool)
      .await?,
    )
  }

  pub fn update_last_accessed(&self, pg_pool: &Pool<Postgres>) {
    let deployment_id = self.id;
    let pool = pg_pool.clone();
    tokio::spawn(async move {
      let result = sqlx::query!(
        "UPDATE deployment SET last_accessed_at = $1 WHERE id = $2",
        Utc::now(),
        deployment_id
      )
      .execute(&pool)
      .await;

      match result {
        Ok(rows) => {
          if rows.rows_affected() > 0 {
            debug!("Updated last_accessed_at for deployment: {}", deployment_id);
          } else {
            warn!("No deployment found with id: {}", deployment_id);
          }
        }
        Err(e) => {
          error!(
            "Failed to update last_accessed_at for deployment {}: {}",
            deployment_id, e
          );
        }
      }
    });
  }
}

impl Deployment {
  /// Returns the deployment formated container image tag
  ///
  /// Format structure: {owner_id}/{service_id}:{deployment_id}
  fn image_tag(&self) -> String {
    format!("{}/{}:{}", self.owner_id, self.service_id, self.id)
  }

  /// Finds an available TCP port on the host in the range 10000-20000
  pub fn find_available_host_port() -> anyhow::Result<i16> {
    let mut rng = rand::rng();

    for _ in 0..1000 {
      let port = rng.random_range(10000..=20000);
      if TcpListener::bind(format!("0.0.0.0:{}", port)).is_ok() {
        return Ok(port);
      }
    }
    Err(anyhow!("Failed to find an available port"))
  }
}
