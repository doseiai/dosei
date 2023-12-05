use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt};
use std::error::Error;
use dosei_proto::CronJob;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
    let address = format!("{}:{}", "0.0.0.0", "8844");
    let mut stream = TcpStream::connect(&address).await?;

    // Write the serialized data
    stream.write_all(&serialized_job).await?;
    Ok(())
}
