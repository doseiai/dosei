mod deployment;
mod ping;
mod role;
mod session;
mod user;

use crate::config;
use crate::config::Config;
use crate::container::check_docker_daemon_status;
use anyhow::{anyhow, Context};
use axum::{routing, Extension, Router};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

pub async fn start_server(config: &'static Config) -> anyhow::Result<()> {
  check_docker_daemon_status().await;

  let pool = Pool::<Postgres>::connect(&config.database_url)
    .await
    .context("Failed to connect to Postgres")?;
  sqlx::migrate!().run(&pool).await?;
  let shared_pool = Arc::new(pool);
  config::create_default_user(Arc::clone(&shared_pool), config).await;

  let app = Router::new()
    .route("/ping", routing::get(ping::ping))
    .route(
      "/login",
      routing::post(session::route::login_username_password),
    )
    .route("/logout", routing::delete(session::route::logout))
    .route("/user", routing::get(user::route::user))
    .route("/deploy", routing::post(deployment::route::deploy))
    .layer(TraceLayer::new_for_http())
    .layer(CorsLayer::permissive())
    .layer(Extension(Arc::clone(&shared_pool)))
    .layer(Extension(config));
  let address = config.address();
  let listener = TcpListener::bind(&address)
    .await
    .context("Failed to start server")?;

  print_logo(config);
  tokio::spawn(async move {
    info!("Dosei running on http://{} (Press Ctrl+C to quit)", address);
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

fn print_logo(config: &'static Config) {
  println!(
    "
                  @@@@@@@@@@@.         &@@@@.
            @@@      @@@@  .@@@  @@,    #@@&
          @@%   &@@@#      &@@@         @@          {}
         @@@@@@.      .@@@&          &@@
        @@       *@@@@           %@@@               Running in standalone mode
        @@ .@@@@(            @@@@                   Port {}
                        @@@@  #@@@@
     @@@         .@@@@%  %@@@   @@
  /@@ .&*   @@@@.  (@@@@      @@%
 @@@       ,@@@@@@         @@@                      https://dosei.io
 .@@@@@@@%        @@@@@@@@
  ",
    env!("CARGO_PKG_DESCRIPTION"),
    &config.port,
  );
}
