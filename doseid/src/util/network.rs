use rand::Rng;
use tokio::net::TcpListener;

pub(crate) async fn find_available_port() -> Option<u16> {
  let mut rng = rand::thread_rng();

  for _ in 0..1000 {
    let port = rng.gen_range(10000..=20000);
    if TcpListener::bind(("0.0.0.0", port)).await.is_ok() {
      return Some(port);
    }
  }
  None
}
