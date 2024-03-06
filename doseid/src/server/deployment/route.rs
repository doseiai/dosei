use crate::config::Config;
use axum::extract::Multipart;
use axum::{Extension};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use crate::docker::{build_image_raw};

pub async fn api_deploy(
  pool: Extension<Arc<Pool<Postgres>>>,
  config: Extension<&'static Config>,
  headers: axum::http::HeaderMap,
  mut multipart: Multipart,
) {
  let mut combined_data = Vec::new();
  while let Some(mut field) = multipart.next_field().await.unwrap() {
    let name = field.name().unwrap().to_string();
    let data = field.bytes().await.unwrap();
    combined_data.extend(data.clone());
    println!("Length of `{}` is {} bytes", name, data.len());
  }
  build_image_raw("example", "example", combined_data).await;
  // let session = validate_session(Arc::clone(&pool), &config, headers)
  //   .await
  //   .map_err(|e| e.into_response())?;
}
