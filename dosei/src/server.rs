use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt};
use tokio::time::sleep;
use log::{info};
use dosei_proto::cron_job;
use prost::Message;
use std::time::Duration;

fn start_job_manager() {
    tokio::spawn(async {
        loop {
            info!("loop");
            // read_minute_jobs().await;
            sleep(Duration::from_secs(60)).await;
        }
    });
    info!("Dosei Job Manager initialized");
}

fn start_main_node() {
    tokio::spawn(async {
        let main_address = format!("{}:{}", "0.0.0.0", "8844".parse::<i32>().unwrap() + 10000);
        let listener = TcpListener::bind(&main_address).await.expect("Failed to bind to address");
        loop {
            let (mut socket, _) = listener.accept().await.expect("Failed to accept connection");
            let mut buf = Vec::new(); // buffer for reading data

            // Read data into buffer
            let n = match socket.read_to_end(&mut buf).await {
                Ok(n) => n,
                Err(_) => return,
            };
            if n == 0 {
                return;
            }
            println!("Bytes read: {}", n);

            // Try to deserialize the data into CronJob
            let received_data = match cron_job::CronJob::decode(&*buf) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Failed to decode: {}", e);
                    continue;
                },
            };
            println!("Received: {:?}", received_data); // Log the received data
        }
    });
    info!("Dosei Node main initialized");
}

pub async fn start_server() {
    let address = format!("{}:{}", "0.0.0.0", "8844");
    start_job_manager();
    start_main_node();
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    info!("Dosei running on http://{} (Press CTRL+C to quit", &address);
    let builder = axum::Server::bind(&address.parse().unwrap());
    builder.serve(app.into_make_service()).await.unwrap();
}
