use anyhow::Result;
use bytes::{BufMut, Bytes, BytesMut};
use metainfo::{bendy::decoding::FromBencode, MetaInfo};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub mod client;
pub use client::Client;

pub async fn start(torrent: &str) -> Result<()> {
    println!("Parsing torrent");

    // Read torrent and parse meta_info
    let t = fs::read(torrent).await?;
    let meta_info = MetaInfo::from_bencode(&t).expect("Failed to parse torrent file");

    if let Some(announce) = &meta_info.announce {
        let announce = &announce.0;
        // Get announce url and make sure it's a valid url
        println!("Found announce url: {}", announce.as_str());
        println!("Protocol: {}", announce.scheme());

        // Build the tracker url using ip and port
        let tracker = format!(
            "{}:{}",
            announce.host_str().unwrap(),
            announce.port().unwrap()
        );

        let info_hash = meta_info.info_hash()?;
        let left = meta_info.length();

        println!("Connecting to {}", announce.as_str());

        // Initialize bittorent client
        let client = Client::new(b"-LE0001-").await?;
        println!("{:?}", client.peer_id);

        client.announce(announce.as_str(), info_hash, left).await?;

        return Ok(());

        let connect_res = client.connect_udp(&tracker).await?;

        println!("Sending announce request");

        // Send announce request
        let announce_res = client
            .announce_udp(connect_res.connection_id, info_hash, left)
            .await?;

        let peers = announce_res.peers;
        println!("Found {} peers", peers.len());

        // All the possible messages, see https://wiki.theory.org/BitTorrentSpecification#Messages
        let handshake = build_handshake(meta_info, client.peer_id);

        const KEEP_ALIVE: [u8; 4] = [0, 0, 0, 0];
        const CHOKE: [u8; 5] = [0, 0, 0, 0, 1];
        const UNCHOKE: [u8; 5] = [0, 0, 0, 1, 1];
        const INTERESTED: [u8; 5] = [0, 0, 0, 1, 2];
        const NOT_INTERESTED: [u8; 5] = [0, 0, 0, 1, 3];

        // Create tcp connection
        // If the connection is refused it probably means this peer is no good
        // In a proper client you'd want to connect to as many peers as possible and discard bad ones
        // but for the sake of simplicity I'll connect just to one for now

        println!("Creating tcp stream");
        let mut stream = TcpStream::connect(peers[10]).await?;
        println!("{}", stream.local_addr()?.to_string());

        let mut buffer = BytesMut::with_capacity(65508);
        buffer.resize(65508, 0);

        stream.write(&handshake).await?;
        let n = stream.read(&mut buffer).await?;
        buffer.truncate(n);

        println!("{:?}", &buffer);
        println!("{:?}", &buffer[..]);
        println!("{:?}", &buffer.len());
        println!("{}", std::str::from_utf8(&buffer[1..20])?);
        // stream.read(&mut [0; 128]).await?;
    } else {
        // If no announce url is found it means we should lookup the DHT
        // DHT is a very complicated topic so I won't even try for now
        println!("No announce url found");
    }

    Ok(())
}

fn build_handshake(meta_info: MetaInfo, peer_id: Bytes) -> Bytes {
    let mut handshake = BytesMut::with_capacity(68);
    handshake.put_u8(19); // pstrlen. Always 19 in the 1.0 protocol
    handshake.put(&b"BitTorrent protocol"[..]); // pstr. Always BitTorrent protocol in the 1.0 protocol
    handshake.put_u64(0); // reserved bytes. All current implementations use all zeroes
    handshake.put_slice(&meta_info.info_hash().unwrap()); // torrent info hash
    handshake.put_slice(&peer_id);
    handshake.freeze()
}

fn build_have_message(piece_index: u32) -> Bytes {
    let mut have = BytesMut::with_capacity(9);
    have.put_u32(5);
    have.put_u8(4);
    have.put_u32(piece_index);
    have.freeze()
}

// I have yet to fully understand how this message works but since it's optional I'll look into it later
// fn build_bitfield_message(payload: BytesMut) -> BytesMut {
//     let mut bitfield = BytesMut::with_capacity(14);
//     bitfield.put_u32(u32::try_from(bitfield.len()).unwrap() + 1);
//     bitfield.p
// }

fn build_request_message(index: u32, begin: u32, length: u32) -> Bytes {
    let mut request = BytesMut::with_capacity(17);
    request.put_u32(13);
    request.put_u8(6);

    request.put_u32(index);
    request.put_u32(begin);
    request.put_u32(length);
    request.freeze()
}

fn build_cancel_message(index: u32, begin: u32, length: u32) -> Bytes {
    let mut cancel = BytesMut::with_capacity(17);
    cancel.put_u32(13);
    cancel.put_u8(8);

    cancel.put_u32(index);
    cancel.put_u32(begin);
    cancel.put_u32(length);
    cancel.freeze()
}

fn build_port_message(listen_port: u16) -> Bytes {
    let mut port = BytesMut::with_capacity(7);
    port.put_u32(3);
    port.put_u8(9);

    port.put_u16(listen_port);
    port.freeze()
}

// fn build_request_message(payload: u32) -> BytesMut {
//     let mut request = BytesMut::with_capacity(17);
// }
