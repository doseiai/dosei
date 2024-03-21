use anyhow::anyhow;
use bollard::auth::DockerCredentials;
use gcp_auth::AuthenticationManager;
use std::env;

pub async fn docker_credentials() -> anyhow::Result<DockerCredentials> {
  // TODO: Get this from config
  let container_registry = env::var("CONTAINER_REGISTRY_URL")?;
  if ContainerRegistry::registry_type(&container_registry) == ContainerRegistry::GoogleCloud {
    return gcr_credentials().await;
  }
  Err(anyhow!("{} not supported.", container_registry))
}

async fn gcr_credentials() -> anyhow::Result<DockerCredentials> {
  let authentication_manager = AuthenticationManager::new().await?;
  let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
  let token = authentication_manager.get_token(scopes).await?;
  Ok(DockerCredentials {
    username: Some("oauth2accesstoken".to_string()),
    password: Some(token.as_str().to_string()),
    ..Default::default()
  })
}

#[derive(PartialEq)]
enum ContainerRegistry {
  GitHub,
  GoogleCloud,
  #[allow(clippy::upper_case_acronyms)]
  AWS,
  Unsupported,
}

impl ContainerRegistry {
  fn registry_type(domain: &str) -> ContainerRegistry {
    if domain.contains("ghcr.io") {
      ContainerRegistry::GitHub
    } else if domain.contains("pkg.dev") || domain.contains("gcr.io") {
      ContainerRegistry::GoogleCloud
    } else if domain.contains("amazonaws.com") {
      ContainerRegistry::AWS
    } else {
      ContainerRegistry::Unsupported
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::docker::credentials::ContainerRegistry;

  #[test]
  fn test_registry_type() {
    assert!(ContainerRegistry::registry_type("ghcr.io/doseiai/dosei") == ContainerRegistry::GitHub);
  }
}
