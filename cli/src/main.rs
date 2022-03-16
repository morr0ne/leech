use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the torrent file
    #[clap(short, long)]
    torrent: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let Args { torrent } = Args::parse();

    leech_core::start(&torrent).await?;

    Ok(())
}
