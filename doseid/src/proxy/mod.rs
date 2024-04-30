use bollard::Docker;

pub const PROXY_SERVICE_NAME: &str = "dosei_proxy";

pub(crate) async fn start_proxy() -> anyhow::Result<()> {
  let _docker = Docker::connect_with_socket_defaults()?;

  Ok(())
}

pub(crate) async fn shutdown_proxy() -> anyhow::Result<()> {
  let docker = Docker::connect_with_socket_defaults()?;

  docker.stop_container(PROXY_SERVICE_NAME, None).await?;
  docker.remove_container(PROXY_SERVICE_NAME, None).await?;

  Ok(())
}
