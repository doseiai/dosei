mod config;
mod deployment;
mod docker;
mod git;
mod schema;
mod server;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  if !config.telemetry.is_disabled() {
    config.telemetry.client.as_ref().unwrap().identify().await;
  }
  server::start_server(config).await?;
  Ok(())
}
