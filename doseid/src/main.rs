mod config;
mod container;
mod crypto;
mod proxy;
pub(crate) mod server;

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  server::start_server(config).await?;
  Ok(())
}
