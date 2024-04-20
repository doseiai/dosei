mod ping;

use crate::config::Config;
use anyhow::{anyhow, Context};
use axum::{routing, Extension, Router};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

pub async fn start_server(config: &'static Config) -> anyhow::Result<()> {
  let pool = Pool::<Postgres>::connect(&config.database_url)
    .await
    .context("Failed to connect to Postgres")?;
  let shared_pool = Arc::new(pool);

  let app = Router::new()
    .route("/ping", routing::get(ping::ping))
    .layer(Extension(Arc::clone(&shared_pool)))
    .layer(Extension(config));

  let address = config.address();
  let listener = TcpListener::bind(&address)
    .await
    .context("Failed to start server")?;
  tokio::spawn(async move {
    info!(
      "Dosei Proxy running on http://{} (Press Ctrl+C to quit)",
      address
    );
    axum::serve(listener, app)
      .await
      .expect("Failed start Dosei API");
  });
  signal::ctrl_c()
    .await
    .map_err(|err| anyhow!("Unable to listen for shutdown signal: {}", err))?;
  info!("Gracefully stopping... (Press Ctrl+C again to force)");
  Ok(())
}
