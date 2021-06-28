use anyhow::Result;
use clap::{App, Arg};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize cli application
    let matches = App::new("leech")
        .version("0.1.0")
        .arg(Arg::new("torrent").required(true))
        .get_matches();

    // Get path to the torrent
    let torrent = matches.value_of("torrent").unwrap();

    leech_core::start(&torrent).await?;

    Ok(())
}
