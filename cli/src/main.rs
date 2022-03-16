use clap::Parser;
use color_eyre::eyre::Result;

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
    color_eyre::install()?;

    let Args { torrent } = Args::parse();

    leech_core::start(&torrent).await?;

    Ok(())
}
