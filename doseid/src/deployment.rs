pub mod app;

use crate::config::Config;
use crate::deployment::app::import_dosei_app;
use crate::docker::build_image;
use std::path::Path;

async fn build(folder_path: &Path) {
  let detected_docker_file = dosei_util::package_manager::_resolve_docker(folder_path);
  if detected_docker_file {
    println!("Detected `Dockerfile`");
    let _ = build_image("example", "example", folder_path).await;
    if let Ok(app) = import_dosei_app("example", "example", folder_path).await {
      println!("{:?}", app);
    }
  }
}

pub async fn _build_internal(config: &'static Config, repo_full_name: &str, deployment_id: &str) {
  let file_appender = tracing_appender::rolling::never(".", "my_log.log");
  let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

  let subscriber = tracing_subscriber::fmt().with_writer(non_blocking).finish();

  let _guard = tracing::subscriber::set_default(subscriber);
}

#[cfg(test)]
mod tests {
  use crate::config::Config;
  use crate::deployment::_build_internal;
  use once_cell::sync::Lazy;

  static CONFIG: Lazy<Config> = Lazy::new(|| Config::new().unwrap());

  #[tokio::test]
  async fn test_build() {
    if CONFIG.github_integration.is_some() {
      _build_internal(&CONFIG, "doseiai/api", "test").await;
    }
  }
}
