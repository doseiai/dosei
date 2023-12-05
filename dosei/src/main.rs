use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt};
use std::io;
use std::error::Error;
use dosei_proto::CronJob;
use clap::Parser;
use tokio::io::AsyncWriteExt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address and port of the server to connect to
    #[arg(short, long)]
    connect: Option<String>,
}

async fn start_client(address: String) -> Result<(), Box<dyn Error>> {
    // Create a new CronJob instance
    let job = CronJob {
        id: String::from("123"),
        schedule: String::from("0 5 * * *"),
        entrypoint: String::from("/path/to/script.sh"),
        deployment_id: String::from("456"),
    };

    // Serialize the CronJob instance to JSON
    let serialized_job = serde_json::to_vec(&job)?;

    // Connect to a peer
    let mut stream = TcpStream::connect(&address).await?;

    // Write the serialized data
    stream.write_all(&serialized_job).await?;
    Ok(())
}

async fn process_socket(mut socket: TcpStream) -> io::Result<()> {
    let mut buf = vec![0; 1024]; // buffer for reading data

    // Read data into buffer
    let n = socket.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    println!("{}", n);
    // Try to deserialize the data into ClientData
    let received_data = serde_json::from_slice::<CronJob>(&buf[..n])?;
    println!("Received: {:?}", received_data); // Log the received data

    Ok(())
}

async fn start_server() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8844").await?;

    loop {
        let (socket, _) = listener.accept().await?;
        process_socket(socket).await?;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.connect.is_some() {
        start_client(args.connect.unwrap()).await?;
    } else {
        start_server().await?;
    }
    Ok(())
}
