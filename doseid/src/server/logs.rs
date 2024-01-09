use axum::body::Body;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use home::home_dir;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

pub async fn deployment_logs(Path(params): Path<DeploymentLogParams>) -> impl IntoResponse {
  // testing
  let mut path = home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
  path.push(format!(
    ".dosei/doseid/data/deployments/logs/{}.logs",
    params.deployment_id
  ));

  let file = match tokio::fs::File::open(path).await {
    Ok(file) => file,
    Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
  };
  let stream = ReaderStream::new(file);
  let body = Body::from_stream(stream);
  Ok(body)
}

// pub async fn deployment_logstream(
//   ,
// ) -> Result<Json<Info>, StatusCode> {
//   let cluster_info = Arc::clone(&CLUSTER_INFO);
//   Ok(Json(Info {
//     server: Server {
//       id: config.node_info.id,
//       mode: if config.is_primary() && cluster_info.lock().await.replicas.is_empty() {
//         Mode::STANDALONE
//       } else {
//         Mode::CLUSTER
//       },
//       address: config.address.clone(),
//       version: VERSION.to_string(),
//     },
//   }))
// }

#[derive(Deserialize, Debug)]
pub struct DeploymentLogParams {
  deployment_id: String, // this needs to be UUID
}
