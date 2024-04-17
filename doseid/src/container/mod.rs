use bollard::Docker;
use tracing::error;

pub(crate) async fn check_docker_daemon_status() {
  match Docker::connect_with_socket_defaults() {
    Ok(connection) => match connection.ping().await {
      Ok(_) => {}
      Err(e) => {
        error!("Failed to ping Docker: {}", e);
        std::process::exit(1);
      }
    },
    Err(e) => {
      error!("Failed to connect to Docker: {}", e);
      std::process::exit(1);
    }
  };
}
