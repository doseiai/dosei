use tokio::time::sleep;
use std::time::Duration;

pub fn start_job_manager() {
  tokio::spawn(async {
    loop {
      // read_minute_jobs().await;
      sleep(Duration::from_secs(60)).await;
    }
  });
}
