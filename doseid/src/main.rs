mod config;
mod schema;
mod server;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let subscriber = tracing_subscriber::fmt()
    .with_line_number(true)
    .with_target(true)
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;

  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  if !config.telemetry.is_disabled() {
    config.telemetry.client.as_ref().unwrap().identify().await;
  }
  server::start_server(config).await?;
  Ok(())
}
