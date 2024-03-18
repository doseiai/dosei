use anyhow::anyhow;
use rand::Rng;
use std::net::TcpListener;

// TODO: Make this async
pub(crate) fn find_available_port() -> anyhow::Result<u16> {
  let mut rng = rand::thread_rng();

  for _ in 0..1000 {
    let port = rng.gen_range(10000..=20000);
    if TcpListener::bind(format!("0.0.0.0:{}", port)).is_ok() {
      return Ok(port);
    }
  }
  Err(anyhow!("Failed to find an available port"))
}
