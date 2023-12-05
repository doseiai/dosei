use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt};
use std::io;
use dosei_proto::dosei::cron_job;
use prost::Message;

async fn process_socket(mut socket: TcpStream) -> io::Result<()> {
    let mut buf = Vec::new(); // buffer for reading data

    // Read data into buffer
    let n = socket.read_to_end(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    println!("Bytes read: {}", n);

    // Try to deserialize the data into CronJob
    let received_data = cron_job::CronJob::decode(&*buf)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    println!("Received: {:?}", received_data); // Log the received data

    Ok(())
}

pub(crate) async fn start_server() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8844").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        process_socket(socket).await?;
    }
}
