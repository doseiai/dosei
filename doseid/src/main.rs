mod config;
mod schema;
mod server;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(config::init()?));
  println!("{}", config.telemetry_disabled);
  server::start_server(config).await?;
  Ok(())
}
