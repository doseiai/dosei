mod server;
mod client;
mod config;

use std::error::Error;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  env_logger::init();
  let config: Config = config::init();
  server::start_server(config).await;
  Ok(())
}
