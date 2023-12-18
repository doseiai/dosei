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

type Client = hyper_util::client::legacy::Client<HttpConnector, Body>;

#[tokio::main]
async fn main() {
  let client: Client = hyper_util::client::legacy::Client::<(), ()>::builder(TokioExecutor::new())
    .build(HttpConnector::new());

  let app = Router::new()
    .route("/", any(handler))
    .route("/*path", any(handler))
    .with_state(client);

  let listener = tokio::net::TcpListener::bind("127.0.0.1:8081")
    .await
    .unwrap();
  println!("listening on {}", listener.local_addr().unwrap());
  axum::serve(listener, app).await.unwrap();
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
