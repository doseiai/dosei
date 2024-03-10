use bollard::models::{EventMessage, EventMessageTypeEnum};
use bollard::system::EventsOptions;
use bollard::Docker;
use futures_util::StreamExt;
use std::collections::HashMap;
use tracing::{error, info, warn};

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
              info!("create");
            }
            "start" => {
              info!("start");
            }
            "die" => {
              error!("die");
            }
            (event_action) => {
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
}

pub(crate) fn start_docker_event_listener() {
  tokio::spawn(async move {
    listen_docker_events().await;
  });
}
