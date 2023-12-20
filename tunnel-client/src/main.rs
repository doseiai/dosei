use std::time::Duration;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> io::Result<()> {
  let stream = TcpStream::connect("127.0.0.1:8080").await?;
  let (mut reader, mut writer) = stream.into_split();

  let (tx, mut rx) = mpsc::channel::<String>(32);

  // Task for writing data continuously and processing received messages
  let writer_task = tokio::spawn(async move {
    let keep_alive_message = b"Keep-alive message";
    loop {
      tokio::select! {
            Some(message) = rx.recv() => {
      println!("LOL: {}", message);
            },
            _ = sleep(Duration::from_secs(1)) => {
                if let Err(e) = writer.write_all(keep_alive_message).await {
                    eprintln!("Failed to write to stream: {}", e);
                    break;
                }
            },
        }
    }
  });

  // Task for reading data
  let reader_task = tokio::spawn(async move {
    let mut buffer = [0; 1024];
    loop {
      match reader.read(&mut buffer).await {
        Ok(n) => {
          if n == 0 {
            break;
          }
          let msg = String::from_utf8_lossy(&buffer[..n]);
          println!("{}", msg);
          // Process incoming data, possibly sending to writer task via channel
          // tx.send(...).await.unwrap();
        }
        Err(e) => {
          eprintln!("Failed to read from stream: {}", e);
          break;
        }
      }
    }
  });

  // Wait for both tasks to complete
  let _ = tokio::try_join!(writer_task, reader_task)?;

  Ok(())
}
