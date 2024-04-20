mod domain;
mod ping;

use crate::config::Config;
use anyhow::{anyhow, Context};
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::{routing, Extension, Json, Router};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use serde_json::json;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

pub async fn start_server(config: &'static Config) -> anyhow::Result<()> {
  let pool = Pool::<Postgres>::connect(&config.database_url)
    .await
    .context("Failed to connect to Postgres")?;
  let shared_pool = Arc::new(pool);

  let client: Client = hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
    .build(HttpConnector::new());

  let app = Router::new()
    .route("/ping", routing::get(ping::ping))
    .route("/", any(handler))
    .route("/*path", any(handler))
    .with_state(client)
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

async fn handler(
  pool: Extension<Arc<Pool<Postgres>>>,
  State(client): State<Client>,
  mut req: Request,
) -> Result<Response, StatusCode> {
  let headers = req.headers();
  let host = match headers.get("host") {
    Some(host_header) => host_header.to_str().unwrap_or_default(),
    None => {
      return Ok(
        (
          StatusCode::NOT_FOUND,
          Json(json!({"message": "Deployment not found."})),
        )
          .into_response(),
      )
    }
  };
  let path = req.uri().path();
  let path_query = req
    .uri()
    .path_and_query()
    .map(|v| v.as_str())
    .unwrap_or(path);

  match domain::get_domain(host.to_string(), Arc::clone(&pool)).await {
    None => Ok(
      (
        StatusCode::NOT_FOUND,
        Json(json!({"message": "Deployment not found."})),
      )
        .into_response(),
    ),
    Some(target_port) => {
      let target_service = format!("http://127.0.0.1:{}{}", target_port, path_query);
      info!("Forwarding: {} -> {}", host, target_service);
      *req.uri_mut() = Uri::try_from(target_service).unwrap();
      Ok(
        client
          .request(req)
          .await
          .map_err(|_| StatusCode::BAD_REQUEST)?
          .into_response(),
      )
    }
  }
}
