use tokio::net::TcpStream;
use std::error::Error;
use dosei_proto::CronJob;
use tokio::io::AsyncWriteExt;

pub async fn start_client(address: String) -> Result<(), Box<dyn Error>> {
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
