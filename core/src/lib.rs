#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]

use anyhow::{anyhow, Result};
use bento::FromBencode;
use std::time::Duration;
use tokio::{
    fs,
    io::{AsyncRead, AsyncWrite},
    net::{TcpStream, ToSocketAddrs},
    time::timeout,
};
use tracker::tracker::http::AnnounceRequest;

pub mod client;
pub mod meta_info;
pub mod protocol;
pub mod session;
pub mod utp;

pub use client::Client;
pub use meta_info::MetaInfo;
pub use protocol::*;
use utp::UtpStream;

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

pub struct Connection<S> {
    status: Status,
    wire: Wire<S>,
    fast: bool,
}

pub struct ConnectionBuilder;

impl ConnectionBuilder {
    pub const fn new() -> Self {
        ConnectionBuilder {}
    }

    pub async fn connect_tcp<A: ToSocketAddrs>(
        addr: A,
        info_hash: [u8; 20],
        peer_id: [u8; 20],
    ) -> Result<Connection<TcpStream>> {
        let handshake = Handshake::new([0, 0, 0, 0, 0, 0x10, 0, 0], info_hash, peer_id);
        let stream = TcpStream::connect(addr).await?;
        let (peer_info, wire) = Wire::handshake(handshake, stream).await?;

        Ok(Connection {
            status: Status::new(),
            wire,
            fast: peer_info.fast_extension,
        })
    }

    pub async fn connect_ucp<A: ToSocketAddrs>(
        addr: A,
        info_hash: [u8; 20],
        peer_id: [u8; 20],
    ) -> Result<Connection<UtpStream>> {
        let handshake = Handshake::new([0, 0, 0, 0, 0, 0x10, 0, 0], info_hash, peer_id);
        todo!()
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {}

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
        // If the connection is refused we simply try to connect to another peer
        let mut wire = {
            let mut f = None;

            for peer in peers {
                println!("Connecting to: {}", peer);

                let timeout = timeout(Duration::from_secs(3), TcpStream::connect(peer)).await;

                if let Ok(Ok(stream)) = timeout {
                    let handshake = Handshake::new([0, 0, 0, 0, 0, 0x10, 0, 0], info_hash, peer_id);
                    if let Ok((peer_info, wire)) = Wire::handshake(handshake, stream).await {
                        println!(
                            "Connected to {}",
                            String::from_utf8_lossy(&peer_info.peer_id)
                        );
                        println!(
                            "FAST: {}, DHT: {}, LTEP: {}",
                            peer_info.fast_extension,
                            peer_info.dht_extension,
                            peer_info.extension_protocol
                        );

                        f = Some(wire);
                        break;
                    } else {
                        println!("Failed to handshake with peer");
                        continue;
                    }
                }

                println!("Failed to connect to peer: {}", peer);
            }

            f.ok_or(anyhow!("Failed to find a peer"))?
        };

        let mut status = Status::new();

        while let Some(message) = wire.read_message().await? {
            match message {
                Message::KeepAlive => {}
                Message::Choke => {
                    println!("Peer choking");
                    status.peer_choking = true;
                }
                Message::Unchoke => {
                    println!("Peer stopped choking");
                    status.peer_choking = false;
                }
                Message::Interested => {
                    println!("Peer interested");
                    status.peer_interested = true;
                }
                Message::NotInterested => {
                    println!("Peer not interested");
                    status.peer_interested = false;
                }
                Message::Bitfield(_bitfield) => {
                    println!("Peer sent bitfield")
                }
                Message::Extended { id, payload } => {
                    dbg!(id, &payload);
                    if id == 0 {
                        let handshake = ExtendedHandshake::from_bencode(&payload)?;
                        dbg!(handshake);
                    }
                }
                Message::Unknown { id, payload } => {
                    println!("Uknown message id {}", id)
                }
                _ => {
                    dbg!(message);
                }
            };
        }
    } else {
        // If no announce url is found it means we should lookup the DHT
        // DHT is a very complicated topic so I won't even try for now
        println!("No announce url found");
    }

    Ok(())
}
