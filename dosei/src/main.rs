mod config;
mod server;

use config::Config;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenv().ok();
  env_logger::init();
  let config: Config = config::init();
  server::start_server(&config).await;
  Ok(())
}
