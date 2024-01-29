use crate::config::DEPLOYMENT_LOG_PATH;
use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use home::home_dir;
use serde::Deserialize;
use std::path::PathBuf;
use tokio_util::io::ReaderStream;

pub async fn deployment_logs(Path(params): Path<DeploymentLogParams>) -> impl IntoResponse {
  let mut path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
  path.push(format!(
    "{}/{}.logs",
    DEPLOYMENT_LOG_PATH, params.deployment_id
  ));

  let file = match tokio::fs::File::open(path).await {
    Ok(file) => file,
    Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
  };
  let stream = ReaderStream::new(file);
  let body = Body::from_stream(stream);
  Ok(body)
}

// pub async fn deployment_logstream()

#[derive(Deserialize, Debug)]
pub struct DeploymentLogParams {
  deployment_id: String, // this needs to be UUID
}
