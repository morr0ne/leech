#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]

use anyhow::{bail, Result};
use bytes::BytesMut;
use metainfo::{bento::decode::FromBencode, MetaInfo};

use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub mod client;
pub mod message;

pub use client::Client;
use tracker::tracker::http::AnnounceRequest;

use crate::message::Handshake;

pub async fn start(torrent: &str) -> Result<()> {
    let peer_id = peers::peer_id(b"-LE0001-");
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
            event: None,
            compact: true,
            numwant: None,
        };

        println!("Built announce request");

        let announce_response = client
            .announce(announce.as_str(), &announce_request)
            .await?;

        let peers = announce_response.peers;

        println!("Found {} peers", peers.len());

        let handshake = Handshake {
            reserved_bytes: [0u8; 8],
            info_hash,
            peer_id,
        }
        .into_bytes();

        // Create tcp connection
        // If the connection is refused it probably means this peer is no good
        // In a proper client you'd want to connect to as many peers as possible and discard bad ones
        // but for the sake of simplicity I'll connect just to one for now

        println!("Creating tcp stream");
        let mut stream = TcpStream::connect(peers[10]).await?;
        println!("Connected to {}", stream.peer_addr()?.to_string());

        let mut buffer = BytesMut::with_capacity(65508);
        buffer.resize(65508, 0);

        stream.write(&handshake).await?;
        let n = stream.read(&mut buffer).await?;
        buffer.truncate(n);

        println!("Received {} bytes", n);

        let handshake = Handshake::from_bytes(buffer)?;

        if handshake.info_hash == info_hash {
            println!("Info hash matches")
        }

        println!("peer_id: {}", String::from_utf8_lossy(&handshake.peer_id));
    } else {
        // If no announce url is found it means we should lookup the DHT
        // DHT is a very complicated topic so I won't even try for now
        println!("No announce url found");
    }

    Ok(())
}
