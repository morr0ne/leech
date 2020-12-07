use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use clap::{App, Arg};
use rand::random;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::{
    fs,
    net::{TcpStream, UdpSocket},
};
use url::Url;

pub mod client;
pub mod meta_info;

pub use client::Client;

use meta_info::MetaInfo;

pub const NONE: i32 = 0;
pub const COMPLETED: i32 = 1;
pub const STARTED: i32 = 2;
pub const STOPPED: i32 = 3;

pub const NUM_WANT: i32 = -1;

pub const CHOKE: i32 = 0;
pub const UNCHOKE: i32 = 1;
pub const INTERESTED: i32 = 2;
pub const NOT_INTERESTED: i32 = 3;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize cli application
    let matches = App::new("leech")
        .version("0.1.0")
        .arg(Arg::with_name("torrent").required(true))
        .get_matches();

    // Get path to the torrent
    let torrent = matches.value_of("torrent").unwrap();

    println!("Parsing torrent");

    // Read torrent and parse meta_info
    let t = fs::read(torrent).await?;
    let meta_info: MetaInfo = serde_bencode::from_bytes(&t)?;

    // Get announce tracker and make sure it's a valid url
    let tracker = Url::parse(&meta_info.announce)?;

    // Build the tracker url using ip and port
    let url = format!(
        "{}:{}",
        tracker.host_str().unwrap(),
        tracker.port().unwrap()
    );

    println!("Connecting to {}", &url);

    // Initialize bittorent client
    let client = Client::new().await?;
    let connect_res = client.connect(&url).await?;

    let info_hash: [u8; 20] = meta_info.info_hash()?;

    let peer_id: [u8; 20] = random::<[u8; 20]>();

    let left = match meta_info.info {
        meta_info::Info::SingleFile { length, .. } => length,
        meta_info::Info::MultiFile { ref files, .. } => {
            files.iter().fold(0, |index, file| index + file.length)
        }
    };

    println!("Sending announce request");

    // Send announce request
    let announce_res = client
        .announce(connect_res.connection_id, info_hash, left)
        .await?;

    // Create tcp connection and build handshake
    // If the connection is refused it probably means this peer is no good
    // In a proper client you'd want to connect to as many peers as possible 
    // but for the sake of simplicity I'll connect just to one for now

    let mut stream = TcpStream::connect(announce_res.peers[0]).await?;

    let mut handshake = BytesMut::with_capacity(68);

    handshake.put_u8(19);
    handshake.put(&b"BitTorrent protocol"[..]);
    handshake.put_u32(0);
    handshake.put_u32(0);
    handshake.put_slice(&meta_info.info_hash()?);
    handshake.put_slice(&peer_id);

    Ok(())
}
