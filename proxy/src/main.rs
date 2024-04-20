use crate::config::Config;

mod config;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  server::start_server(config).await?;
  Ok(())
}
