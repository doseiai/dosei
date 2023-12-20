use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
  let listener = TcpListener::bind(("127.0.0.1", 8080)).await?;

  while let Ok((mut inbound, _)) = listener.accept().await {
    let mut outbound = TcpStream::connect("127.0.0.1:8000").await?;

    tokio::spawn(async move {
      let mut inbound_buf = [0; 1024];
      let mut outbound_buf = [0; 1024];

      loop {
        // Read from inbound
        let n = match inbound.read(&mut inbound_buf).await {
          Ok(n) if n == 0 => break,
          Ok(n) => n,
          Err(_) => break,
        };

        // Check for keep-alive message
        let msg = String::from_utf8_lossy(&inbound_buf[..n]);
        if msg.contains("Keep-alive message") {
          println!("Keep-alive message received");
          // Respond to keep-alive message
          let response = "Response to keep-alive";
          if let Err(_) = inbound.write_all(response.as_bytes()).await {
            eprintln!("Failed to send response to keep-alive message");
            break;
          }
        }

        // Forward to outbound
        if let Err(_) = outbound.write_all(&inbound_buf[..n]).await {
          break;
        }

        // Now read from outbound and send back to inbound if needed
        // Similar logic to the above can be implemented here
        // ...
      }
    });
  }

  Ok(())
}
