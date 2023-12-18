use anyhow::Context;
use axum::{
  body::Body,
  extract::{Request, State},
  http::uri::Uri,
  response::{IntoResponse, Response},
  routing::any,
  Router,
};
use dotenv::dotenv;
use hyper::StatusCode;
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use log::info;
use std::env;
use tokio::net::TcpListener;

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  dotenv().ok();
  if env::var("RUST_LOG").is_err() {
    env::set_var("RUST_LOG", "info");
  }
  env_logger::init();
  let client: Client = hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
    .build(HttpConnector::new());

  let app = Router::new()
    .route("/", any(handler))
    .route("/*path", any(handler))
    .with_state(client);

  let address = "127.0.0.1:8081";
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
