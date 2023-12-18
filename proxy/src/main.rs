mod config;

use crate::config::Config;
use anyhow::Context;
use axum::{
  body::Body,
  extract::{Request, State},
  http::uri::Uri,
  response::{IntoResponse, Response},
  routing::any,
  Router,
};
use hyper::StatusCode;
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use log::info;
use tokio::net::TcpListener;

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(config::init()?));
  let client: Client = hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
    .build(HttpConnector::new());

  let app = Router::new()
    .route("/", any(handler))
    .route("/*path", any(handler))
    .with_state(client);

  let address = config.address.to_string();
  let listener = TcpListener::bind(&address)
    .await
    .context("Failed to start server")?;
  info!(
    "Dosei Proxy running on http://{} (Press CTRL+C to quit",
    address
  );
  axum::serve(listener, app).await?;
  Ok(())
}

async fn handler(State(client): State<Client>, mut req: Request) -> Result<Response, StatusCode> {
  let path = req.uri().path();
  let path_query = req
    .uri()
    .path_and_query()
    .map(|v| v.as_str())
    .unwrap_or(path);

  let uri = format!("http://127.0.0.1:8000{}", path_query);

  *req.uri_mut() = Uri::try_from(uri).unwrap();

  Ok(
    client
      .request(req)
      .await
      .map_err(|_| StatusCode::BAD_REQUEST)?
      .into_response(),
  )
}
