use tokio::net::TcpStream;
use std::error::Error;
use dosei_proto::cron_job;
use tokio::io::AsyncWriteExt;
use prost::Message;

pub async fn start_client(address: String) -> Result<(), Box<dyn Error>> {
    // Create a new CronJob instance
    let job = cron_job::CronJob {
        id: String::from("123"),
        schedule: String::from("0 5 * * *"),
        entrypoint: String::from("/path/to/script.sh"),
        deployment_id: String::from("456"),
    };

    // Calculate the serialized size of the CronJob
    let mut buf = Vec::with_capacity(job.encoded_len());

    // Serialize the CronJob instance to a buffer
    job.encode(&mut buf)?;

    // Connect to a peer
    let mut stream = TcpStream::connect(&address).await?;

    // Write the serialized data
    stream.write_all(&buf).await?;
    Ok(())
}
