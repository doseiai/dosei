mod config;

use crate::config::Config;
use anyhow::Context;
use axum::response::Redirect;
use axum::{
  body::Body,
  extract::{Request, State},
  http::uri::Uri,
  response::{IntoResponse, Response},
  routing::any,
  Extension, Router,
};
use hyper::StatusCode;
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use log::info;
use mongodb::bson::{doc, Bson, Document};
use mongodb::Database;
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(config::init()?));
  let client_options = mongodb::options::ClientOptions::parse(env::var("MONGODB_URL")?).await?;
  let client = mongodb::Client::with_options(client_options).unwrap();
  let db: Database = client.database("fast");
  let shared_db = Arc::new(db);
  let client: Client = hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
    .build(HttpConnector::new());

  let app = Router::new()
    .route("/", any(handler))
    .route("/*path", any(handler))
    .with_state(client)
    .layer(Extension(Arc::clone(&shared_db)));

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

async fn handler(
  db: Extension<Arc<Database>>,
  State(client): State<Client>,
  mut req: Request,
) -> Result<Response, StatusCode> {
  let headers = req.headers();
  let host = match headers.get("host") {
    Some(host_header) => host_header.to_str().unwrap_or_default(),
    None => return Ok(Redirect::temporary("https://dosei.ai").into_response()),
  };
  let path = req.uri().path();
  let path_query = req
    .uri()
    .path_and_query()
    .map(|v| v.as_str())
    .unwrap_or(path);
  let collection = db.collection::<Document>("domains");
  let filter = doc! {"name": host };
  match collection.find_one(filter, None).await {
    Ok(Some(document)) => {
      if let Some(service_id) = document.get("service_id") {
        if service_id == &Bson::Null {
          return Ok(Redirect::temporary("https://dosei.ai").into_response());
        } else {
          let uri = format!(
            "http://{}.default.svc.cluster.local/{}",
            service_id, path_query
          );
          *req.uri_mut() = Uri::try_from(uri).unwrap();
          // TODO: Trigger proxy pass event
          Ok(
            client
              .request(req)
              .await
              .map_err(|_| StatusCode::BAD_REQUEST)?
              .into_response(),
          )
        }
      } else {
        return Ok(Redirect::temporary("https://dosei.ai").into_response());
      }
    }
    Ok(None) => return Ok(Redirect::temporary("https://dosei.ai").into_response()),
    Err(_) => return Ok(Redirect::temporary("https://dosei.ai").into_response()),
  }
}
