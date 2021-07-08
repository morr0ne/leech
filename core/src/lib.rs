#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]

use anyhow::Result;
use bytes::BytesMut;
use metainfo::{bendy::decoding::FromBencode, MetaInfo};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub mod client;
pub mod utils;

pub use client::Client;
pub use utils::{messages::build_handshake, peer_id};

use client::AnnounceRequest;

pub async fn start(torrent: &str) -> Result<()> {
    let peer_id = peer_id(b"-LE0001-");
    println!("Peer id: {:?}", String::from_utf8_lossy(&peer_id[..]));

    let client = Client::new().await?;

    println!("Parsing torrent");
    let t = fs::read(torrent).await?;
    let meta_info = MetaInfo::from_bencode(&t).expect("Failed to parse torrent file");

    if let Some(announce) = &meta_info.announce {
        println!("Found announce url: {}", announce.as_str());

        let info_hash = meta_info.info_hash()?;
        let left = meta_info.length();

        let announce_request = AnnounceRequest {
            info_hash,
            peer_id,
            ip: None,
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left,
        };

        let peers = client.announce(announce, &announce_request).await?;

        println!("Found {} peers", peers.len());

        // All the possible messages, see https://wiki.theory.org/BitTorrentSpecification#Messages
        let handshake = build_handshake(&info_hash, &peer_id);

        // Create tcp connection
        // If the connection is refused it probably means this peer is no good
        // In a proper client you'd want to connect to as many peers as possible and discard bad ones
        // but for the sake of simplicity I'll connect just to one for now

        println!("Creating tcp stream");
        let mut stream = TcpStream::connect(peers[30]).await?;
        println!("Connected to {}", stream.peer_addr()?.to_string());

        let mut buffer = BytesMut::with_capacity(65508);
        buffer.resize(65508, 0);

        stream.write(&handshake).await?;
        let n = stream.read(&mut buffer).await?;
        buffer.truncate(n);

        println!("{:?}", &buffer.len());
        println!("{}", String::from_utf8_lossy(&buffer[1..20]));
        println!("{:?}", &buffer[..]);
    } else {
        // If no announce url is found it means we should lookup the DHT
        // DHT is a very complicated topic so I won't even try for now
        println!("No announce url found");
    }

    Ok(())
}
