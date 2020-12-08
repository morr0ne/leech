use anyhow::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use rand::random;
use std::convert::TryFrom;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::{
    fs,
    net::{TcpStream, UdpSocket},
    prelude::*,
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

pub async fn start(torrent: &str) -> Result<()> {
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

    let peers = announce_res.peers;

    // Create tcp connection
    // If the connection is refused it probably means this peer is no good
    // In a proper client you'd want to connect to as many peers as possible
    // but for the sake of simplicity I'll connect just to one for now

    // let mut stream = TcpStream::connect(peers[0]).await?;

    // Build the handshake, this is the same for every peer
    let mut handshake = BytesMut::with_capacity(68);

    handshake.put_u8(19); // pstrlen. Always 19 in the 1.0 protocol
    handshake.put(&b"BitTorrent protocol"[..]); // pstr. Always BitTorrent protocol in the 1.0 protocol
    handshake.put_u64(0); // reserved bytes. All current implementations use all zeroes
    handshake.put_slice(&meta_info.info_hash()?); // torrent info hash
    handshake.put_slice(&peer_id); // Unique peer_id, in theory it can always be different

    // All the possible messages, see https://wiki.theory.org/BitTorrentSpecification#Messages

    let mut keep_alive = BytesMut::with_capacity(4);
    keep_alive.put_u32(0);

    let mut choke = BytesMut::with_capacity(5);
    choke.put_u32(1);
    choke.put_u8(0);

    let mut unchoke = BytesMut::with_capacity(5);
    unchoke.put_u32(1);
    unchoke.put_u8(1);

    let mut interested = BytesMut::with_capacity(5);
    interested.put_u32(1);
    interested.put_u8(2);

    let mut not_interested = BytesMut::with_capacity(5);
    not_interested.put_u32(1);
    not_interested.put_u8(3);

    Ok(())
}

fn build_have_message(payload: i32) -> BytesMut {
    let mut have = BytesMut::with_capacity(9);
    have.put_u32(5);
    have.put_u8(4);
    have.put_i32(payload);
    have
}

// I have yet to fully understand how this message works but since it's optional I'll look into it later
// fn build_bitfield_message(payload: BytesMut) -> BytesMut {
//     let mut bitfield = BytesMut::with_capacity(14);
//     bitfield.put_u32(u32::try_from(bitfield.len()).unwrap() + 1);
//     bitfield.p
// }

// fn build_request_message(payload: i32) -> BytesMut {
//     let mut request = BytesMut::with_capacity(17);
// }
