use bollard::auth::DockerCredentials;
use bollard::image::PushImageOptions;
use bollard::Docker;
use gcp_auth::AuthenticationManager;
use std::default::Default;

pub async fn gcr_credentials() -> DockerCredentials {
  let authentication_manager = AuthenticationManager::new().await.unwrap();
  let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
  let token = authentication_manager.get_token(scopes).await.unwrap();
  DockerCredentials {
    username: Some("oauth2accesstoken".to_string()),
    password: Some(token.as_str().to_string()),
    ..Default::default()
  }
}

async fn push_image(name: &str, tag: &str) {
  let docker = Docker::connect_with_socket_defaults().unwrap();
  let _ = docker.push_image(
    name,
    Some(PushImageOptions { tag }),
    Some(gcr_credentials().await),
  );
}
