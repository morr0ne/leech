#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![feature(option_result_unwrap_unchecked)]
#![feature(generic_const_exprs)]

use anyhow::Result;
use metainfo::{bento::FromBencode, MetaInfo};
use tokio::fs;

use tracker::tracker::http::AnnounceRequest;

pub mod client;
pub mod message;
pub mod session;
pub mod wire;

pub use client::Client;
pub use message::Message;
pub use wire::Wire;

pub async fn start(torrent: &str) -> Result<()> {
    let peer_id = peers::peer_id(b"LE", b"0001");
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

        // Create tcp connection
        // If the connection is refused it probably means this peer is no good
        // In a proper client you'd want to connect to as many peers as possible and discard bad ones
        // but for the sake of simplicity I'll connect just to one for now

        // Connect to peer
        println!("Creating tcp stream");
        let peer = peers[13];
        println!("Connecting to {}", peer.to_string());

        let (mut wire, remote_peer_id) = Wire::connect(peer, info_hash, peer_id).await?;
        println!("Connected to {}", String::from_utf8_lossy(&remote_peer_id));

        // tokio::spawn(async move {
        //     while let Some(message) = wire.message_receiver.recv().await {
        //         let message = message.unwrap();
        //         match message {
        //             Message::Unchoke => {
        //                 println!("Peer stopped choking")
        //             }
        //             _ => {
        //                 dbg!(message);
        //             }
        //         };
        //     }
        // });

        // wire.interested().await?;
        wire.have(12).await?;

        // socket_write.write_all(&handshake).await?;
        // socket_write.write_all(&Message::INTERESTED).await?;

        // handle.await?;
    } else {
        // If no announce url is found it means we should lookup the DHT
        // DHT is a very complicated topic so I won't even try for now
        println!("No announce url found");
    }

    Ok(())
}
