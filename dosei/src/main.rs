mod server;
mod config;
mod schema;

use std::error::Error;
use dotenv::dotenv;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  dotenv().ok();
  env_logger::init();
  let config: Config = config::init();
  server::start_server(&config).await;
  Ok(())
}
