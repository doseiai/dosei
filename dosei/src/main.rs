mod server;
mod client;

use std::error::Error;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address and port of the server to connect to
    #[arg(short, long)]
    connect: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    if args.connect.is_some() {
        client::start_client(args.connect.unwrap()).await?;
    } else {
        server::start_server().await?;
    }
    Ok(())
}
