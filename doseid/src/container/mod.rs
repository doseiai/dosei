use bollard::container::ListContainersOptions;
use bollard::models::{ContainerSummary, EventMessage, EventMessageTypeEnum};
use bollard::system::EventsOptions;
use bollard::Docker;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

pub struct Container;

impl Container {
  pub async fn check_docker_daemon_status() {
    match Docker::connect_with_socket_defaults() {
      Ok(connection) => {
        if let Err(e) = connection.ping().await {
          error!("Failed to ping Docker: {}", e);
          std::process::exit(1);
        }
      }
      Err(e) => {
        error!("Failed to connect to Docker: {}", e);
        std::process::exit(1);
      }
    };
  }

  pub async fn start_monitoring_server() -> anyhow::Result<()> {
    info!("DoseiD Container Monitoring Service Running");
    tokio::spawn(async move {
      let mut interval = interval(Duration::from_secs(60));
      loop {
        interval.tick().await;
        for container in Self::get_running_containers().await.unwrap() {
          info!(
            "Container: ID={}, Names={:?}, Image={}, Status={}, State={}, Created={}",
            container.id.as_deref().unwrap_or("N/A"),
            container
              .names
              .as_ref()
              .map(|names| names.join(", "))
              .unwrap_or_else(|| "N/A".to_string()),
            container.image.as_deref().unwrap_or("N/A"),
            container.status.as_deref().unwrap_or("N/A"),
            container.state.as_deref().unwrap_or("N/A"),
            container.created.unwrap_or(0)
          );
        }
      }
    });
    Ok(())
  }

  pub async fn start_event_listener() -> anyhow::Result<()> {
    info!("DoseiD Docker Event Listener Service Running");
    tokio::spawn(async move {
      let docker = Docker::connect_with_socket_defaults().unwrap();
      let mut stream = docker.events(Some(EventsOptions {
        filters: HashMap::from([("type", vec!["container"])]),
        ..Default::default()
      }));
      while let Some(event_result) = stream.next().await {
        match event_result {
          Ok(event) => {
            let event: EventMessage = event;
            match event.typ {
              Some(EventMessageTypeEnum::CONTAINER) => match event.action.clone().unwrap().as_str()
              {
                "create" => {
                  let attributes = &event.actor.clone().unwrap().attributes.unwrap();
                  let name = attributes
                    .get("name")
                    .unwrap_or(&"unknown".to_string())
                    .clone();
                  info!("Container created: {}", name);
                }
                "start" => {
                  let attributes = &event.actor.clone().unwrap().attributes.unwrap();
                  let name = attributes
                    .get("name")
                    .map(|s| s.as_str())
                    .unwrap_or("unknown");
                  let image = attributes
                    .get("image")
                    .map(|s| s.as_str())
                    .unwrap_or("unknown");
                  // Fix: Store the cloned actor in a binding first
                  let actor = event.actor.clone().unwrap();
                  let id = actor.id.as_deref().unwrap_or("unknown");
                  info!(
                    "Container started - Name: {}, Image: {}, ID: {}",
                    name, image, id
                  );
                }
                "die" => {
                  let attributes = &event.actor.clone().unwrap().attributes.unwrap();
                  let name = attributes
                    .get("name")
                    .map(|s| s.as_str())
                    .unwrap_or("unknown");
                  let image = attributes
                    .get("image")
                    .map(|s| s.as_str())
                    .unwrap_or("unknown");
                  let exit_code = attributes
                    .get("exitCode")
                    .map(|s| s.as_str())
                    .unwrap_or("unknown");
                  // Fix: Store the cloned actor in a binding first
                  let actor = event.actor.clone().unwrap();
                  let id = actor.id.as_deref().unwrap_or("unknown");
                  error!(
                    "Container stopped - Name: {}, Image: {}, Exit Code: {}, ID: {}",
                    name, image, exit_code, id
                  );

                }
                event_action => {
                  warn!("Unhandled container event action: {}", event_action);
                }
              },
              Some(EventMessageTypeEnum::BUILDER) => {
                warn!("Unhandled Docker builder events");
              }
              _ => {}
            }
          }
          Err(e) => error!("Docker streaming failed: {:?}", e),
        }
      }
    });
    Ok(())
  }

  async fn get_running_containers() -> anyhow::Result<Vec<ContainerSummary>> {
    let docker = Docker::connect_with_socket_defaults()?;
    let containers = docker
      .list_containers(Some(ListContainersOptions {
        all: false,
        limit: None,
        size: false,
        filters: HashMap::from([("status".to_string(), vec!["running".to_string()])]),
      }))
      .await?;
    Ok(containers)
  }
}
