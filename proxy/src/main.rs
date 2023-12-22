//! WARNING! Unstable Release!
//! This is a work in progress for the migration of our internal proxy to Rust.
//! Expect breaking changes. Use at your own risk.
//!
//! Dosei Proxy
//!
//! Built to run on Dosei K8S Cluster with MongoDB.
//! Currently WIP.
//! TODO:
//! - Implement Redis for Caching
//! - Migrate to Postgres
//! - Run on Dosei Engine
//! - Move /health to only check for internal traffic
//! - Implement events: onProxyPassEvent

mod config;

use crate::config::Config;
use anyhow::Context;
use axum::response::Redirect;
use axum::routing::get;
use axum::{
  body::Body,
  extract::{Request, State},
  http::uri::Uri,
  response::{IntoResponse, Response},
  routing::any,
  Extension, Router,
};
use cached::{Cached, TimedCache};
use hyper::StatusCode;
use hyper_util::{client::legacy::connect::HttpConnector, rt::TokioExecutor};
use mongodb::bson::{doc, Bson, Document};
use mongodb::Database;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::info;

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));
  let client_options = mongodb::options::ClientOptions::parse(&config.mongo_url).await?;
  let client = mongodb::Client::with_options(client_options)?;
  client
    .database("admin")
    .run_command(doc! {"ping": 1}, None)
    .await?;
  info!("Successfully connected to MongoDB");
  let shared_mongo_client = Arc::new(client);
  let client: Client = hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
    .build(HttpConnector::new());

  let app = Router::new()
    .route("/health", get(health))
    .route("/", any(handler))
    .route("/*path", any(handler))
    .with_state(client)
    .layer(Extension(Arc::clone(&shared_mongo_client)));

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

async fn health(mongo_client: Extension<Arc<mongodb::Client>>) -> Result<Response, StatusCode> {
  // TODO: Fix, not sure wtf but not working prod
  // let db: Database = mongo_client.database("admin");
  // match db.run_command(doc! {"ping": 1}, None).await {
  //   Ok(document) => {
  //     if let Ok(ok_value) = document.get_f64("ok") {
  //       if (ok_value - 1.0).abs() < f64::EPSILON {
  //         return Ok((StatusCode::OK, "OK").into_response());
  //       }
  //     }
  //     Err(StatusCode::INTERNAL_SERVER_ERROR)
  //   }
  //   Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  // }
  Ok((StatusCode::OK, "OK").into_response())
}

async fn handler(
  mongo_client: Extension<Arc<mongodb::Client>>,
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
  match get_domain(mongo_client, host.to_string()).await {
    None => Ok(Redirect::temporary("https://dosei.ai").into_response()),
    Some(service_id) => {
      let uri = format!(
        "http://{}.default.svc.cluster.local{}",
        service_id, path_query
      );
      info!("Forwarding: {} -> {}", host, uri);
      *req.uri_mut() = Uri::try_from(uri).unwrap();
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

async fn get_domain(mongo_client: Extension<Arc<mongodb::Client>>, host: String) -> Option<String> {
  let domains_cache = Arc::clone(&DOMAINS_CACHE);
  {
    let mut cache = domains_cache.lock().await;
    if let Some(value) = cache.cache_get(&host) {
      let service_id = value.clone();
      return cache.cache_set(host, service_id);
    }
  }

  let db: Database = mongo_client.database("fast");
  let collection = db.collection::<Document>("domains");

  match collection.find_one(doc! {"name": &host }, None).await {
    Ok(Some(document)) => match document.get("service_id") {
      Some(Bson::String(service_id)) => {
        {
          let mut cache = domains_cache.lock().await;
          cache.cache_set(host, service_id.clone());
        }
        Some(service_id.clone())
      }
      _ => None,
    },
    _ => None,
  }
}

static DOMAINS_CACHE: Lazy<Arc<Mutex<TimedCache<String, String>>>> = Lazy::new(|| {
  let cache = TimedCache::with_lifespan(120);
  Arc::new(Mutex::new(cache))
});
