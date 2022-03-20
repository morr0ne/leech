#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]

use bento::FromBencode;
use bitvec::prelude::BitVec;
use color_eyre::eyre::{eyre, Result};
use std::time::Duration;
use tokio::{
    fs,
    io::{AsyncRead, AsyncWrite},
    net::{TcpStream, ToSocketAddrs},
    time::timeout,
};
use tracing::info;
use tracker::tracker::http::AnnounceRequest;

pub mod client;
pub mod connection;
pub mod meta_info;
pub mod protocol;
pub mod session;
pub mod utp;

pub use client::Client;
pub use meta_info::MetaInfo;
pub use protocol::*;
use utp::UtpStream;

use crate::session::{Session, SessionBuilder};

pub struct Status {
    /// Are we are choking the remote peer?
    pub am_choking: bool,
    /// Are we are interested the remote peer?
    pub am_interested: bool,
    /// Is the remote peer choking us?
    pub peer_choking: bool,
    /// Is the remote peer interested in us?
    pub peer_interested: bool,
}

impl Status {
    pub fn new() -> Self {
        Self {
            am_choking: true,
            am_interested: false,
            peer_choking: true,
            peer_interested: false,
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn start(path: &str) -> Result<()> {
    // info!("Parsing torrent");
    // let torrent = fs::read(path).await?;
    // let meta_info = MetaInfo::from_bencode(&torrent).expect("Failed to parse torrent file");

    // let mut session = Session::builder().keep_alive_interval().connect().await?;

    // while let Some(piece) = session.next_piece().await? {}

    // return Ok(());
    let peer_id = peers::peer_id(b"LE", b"0001");
    info!("Peer id: {:?}", String::from_utf8_lossy(&peer_id[..]));

    let client = Client::new().await?;

    info!("Parsing torrent");
    let t = fs::read(path).await?;
    let meta_info = MetaInfo::from_bencode(&t).expect("Failed to parse torrent file");

    if let Some(announce) = &meta_info.announce {
        info!("Found announce url: {}", announce.as_str());
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

        info!("Built announce request");

        let announce_response = client
            .announce(announce.as_str(), &announce_request)
            .await?;

        let peers = announce_response.peers;

        info!("Found {} peers", peers.len());

        // Create tcp connection
        // If the connection is refused we simply try to connect to another peer
        let mut wire = {
            let mut f = None;

            for peer in peers {
                info!("Connecting to: {}", peer);

                let timeout = timeout(Duration::from_secs(3), TcpStream::connect(peer)).await;

                if let Ok(Ok(stream)) = timeout {
                    let handshake = Handshake::new([0, 0, 0, 0, 0, 0x10, 0, 0], info_hash, peer_id);
                    if let Ok((peer_info, wire)) = Wire::handshake(handshake, stream).await {
                        info!(
                            "Connected to {}",
                            String::from_utf8_lossy(&peer_info.peer_id)
                        );
                        info!(
                            "FAST: {}, DHT: {}, LTEP: {}",
                            peer_info.fast_extension,
                            peer_info.dht_extension,
                            peer_info.extension_protocol
                        );

                        f = Some(wire);
                        break;
                    } else {
                        info!("Failed to handshake with peer");
                        continue;
                    }
                }

                info!("Failed to connect to peer: {}", peer);
            }

            f.ok_or(eyre!("Failed to find a peer"))?
        };

        let handle = tokio::spawn(async move {
            let mut status = Status::new();

            while let Some(message) = wire.read_message().await? {
                match message {
                    Message::KeepAlive => {}
                    Message::Choke => {
                        info!("Peer choking");
                        status.peer_choking = true;
                    }
                    Message::Unchoke => {
                        info!("Peer stopped choking");
                        status.peer_choking = false;
                        wire.write_message(Message::have(0)).await?;
                    }
                    Message::Interested => {
                        info!("Peer interested");
                        status.peer_interested = true;
                    }
                    Message::NotInterested => {
                        info!("Peer not interested");
                        status.peer_interested = false;
                    }
                    Message::Bitfield(_bitfield) => {
                        info!("Peer sent bitfield");
                        wire.write_message(Message::Bitfield(BitVec::EMPTY)).await?;
                    }
                    Message::Extended { id, payload } => {
                        dbg!(id, &payload);
                        if id == 0 {
                            let handshake = ExtendedHandshake::from_bencode(&payload)?;
                            dbg!(handshake);
                        }
                    }
                    Message::Unknown { id, payload } => {
                        info!("Uknown message id {}", id)
                    }
                    _ => {
                        dbg!(message);
                    }
                };
            }

            Ok::<(), color_eyre::eyre::Error>(())
        });

        handle.await??
    } else {
        // If no announce url is found it means we should lookup the DHT
        // DHT is a very complicated topic so I won't even try for now
        info!("No announce url found");
    }

    Ok(())
}
