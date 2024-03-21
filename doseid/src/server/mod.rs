mod app;
mod certificate;
mod cluster;
mod cron;
mod deployment;
mod domain;
mod info;
pub(crate) mod integration;
mod logs;
mod ping;
mod project;
mod secret;
mod session;
mod token;
mod user;

use anyhow::Context;
use sqlx::postgres::Postgres;
use sqlx::Pool;
use std::sync::Arc;

use crate::config::Config;
use crate::docker;
use crate::server::app::shutdown_app;
use axum::{routing, Extension, Router};
use bollard::Docker;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info};

pub async fn start_server(config: &'static Config) -> anyhow::Result<()> {
  check_docker_daemon_status().await;

  if config.console {
    app::start_app().await.context("Failed to start App")?;
    info!("[Integrations] Enabling console");
  }

  let pool = Pool::<Postgres>::connect(&config.database_url)
    .await
    .context("Failed to connect to Postgres")?;
  sqlx::migrate!().run(&pool).await?;
  let shared_pool = Arc::new(pool);
  info!("Successfully connected to Postgres");

  cluster::start_cluster(config)?;
  cron::start_job_manager(config, Arc::clone(&shared_pool));
  docker::event::start_docker_event_listener();
  let app = Router::new()
    .route("/tokens", routing::get(token::route::api_get_tokens))
    .route("/tokens", routing::post(token::route::api_set_token))
    .route(
      "/tokens/:token_id",
      routing::delete(token::route::api_delete_token),
    )
    .route("/envs/:owner_id", routing::post(secret::api_set_envs))
    .route(
      "/envs/:owner_id/:project_id",
      routing::post(secret::api_set_envs),
    )
    .route("/envs/:owner_id", routing::get(secret::api_get_envs))
    .route(
      "/envs/:owner_id/:project_id",
      routing::get(secret::api_get_envs),
    )
    .route(
      "/certificate",
      routing::post(certificate::route::api_new_certificate),
    )
    .route(
      "/.well-known/acme-challenge/:token",
      routing::get(certificate::route::api_http01_challenge),
    )
    .route("/cron-jobs", routing::post(cron::route::api_create_job))
    .route("/cron-jobs", routing::get(cron::route::api_get_cron_jobs))
    .route(
      "/unstable/integration/github/events",
      routing::post(integration::github::route::api_integration_github_events),
    )
    .route(
      "/auth/github/cli",
      routing::get(session::route::api_auth_github_cli),
    )
    .route("/deploy", routing::post(deployment::route::api_deploy))
    .route("/auth/logout", routing::delete(session::route::api_logout))
    .route("/projects/clone", routing::post(project::api_new_project))
    .route("/user", routing::get(user::route::api_get_user))
    .route("/info", routing::get(info::api_info))
    .route("/ping", routing::get(ping::api_ping))
    .route(
      "/deployments/:deployment_id/logs",
      routing::get(logs::deployment_logs),
    )
    // .route(
    //   "/deployments/:deployment_id/logs/stream",
    //   routing::get(logs::deployment_logstream),
    // )
    .layer(Extension(Arc::clone(&shared_pool)))
    .layer(Extension(config));
  let address = config.address.to_string();
  let listener = TcpListener::bind(&address)
    .await
    .context("Failed to start server")?;
  println!(
    "
                  @@@@@@@@@@@.         &@@@@.
            @@@      @@@@  .@@@  @@,    #@@&
          @@%   &@@@#      &@@@         @@          {}
         @@@@@@.      .@@@&          &@@
        @@       *@@@@           %@@@               Running in standalone mode
        @@ .@@@@(            @@@@                   Port {}
                        @@@@  #@@@@                 Node Port {}
     @@@         .@@@@%  %@@@   @@
  /@@ .&*   @@@@.  (@@@@      @@%
 @@@       ,@@@@@@         @@@                      https://dosei.io
 .@@@@@@@%        @@@@@@@@
  ",
    env!("CARGO_PKG_DESCRIPTION"),
    &config.address.port,
    &config.node_info.address.port
  );
  tokio::spawn(async move {
    info!("Dosei running on http://{} (Press CTRL+C to quit", address);
    axum::serve(listener, app)
      .await
      .expect("Failed start Dosei API");
  });
  match signal::ctrl_c().await {
    Ok(()) => {
      shutdown_app().await?;
    }
    Err(err) => error!("Unable to listen for shutdown signal: {}", err),
  }
  Ok(())
}

async fn check_docker_daemon_status() {
  match Docker::connect_with_socket_defaults() {
    Ok(connection) => match connection.ping().await {
      Ok(_) => info!("Successfully connected to Docker Daemon"),
      Err(e) => {
        error!("Failed to ping Docker: {}", e);
        std::process::exit(1);
      }
    },
    Err(e) => {
      error!("Failed to connect to Docker: {}", e);
      std::process::exit(1);
    }
  };
}
