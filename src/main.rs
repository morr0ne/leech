use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use rand::random;
use tokio::fs;
use tokio::net::UdpSocket;
use url::Url;

pub mod meta_info;

use meta_info::MetaInfo;

pub const CONNECT: i32 = 0;
pub const ANNOUNCE: i32 = 1;

pub const NONE: i32 = 0;
pub const COMPLETED: i32 = 1;
pub const STARTED: i32 = 2;
pub const STOPPED: i32 = 3;

pub const NUM_WANT: i32 = -1;

#[tokio::main]
async fn main() -> Result<()> {
    let t = fs::read("manjaro.torrent").await?;

    let meta_info: MetaInfo = serde_bencode::from_bytes(&t)?;
    let tracker = Url::parse(&meta_info.announce)?;

    let url = format!(
        "{}:{}",
        tracker.host_str().unwrap(),
        tracker.port().unwrap()
    );

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(&url).await?;

    const PROTOCOL_ID: i64 = 0x41727101980;
    let transaction_id = random::<i32>();

    let mut connect_req = BytesMut::with_capacity(16);

    connect_req.put_i64(PROTOCOL_ID);
    connect_req.put_i32(CONNECT);
    connect_req.put_i32(transaction_id);

    socket.send(&connect_req).await?;

    let mut connect_res = BytesMut::with_capacity(65508);
    connect_res.resize(65508, 0);

    let n = socket.recv(&mut connect_res).await?;

    println!("Received {} bytes", n);

    let _action = connect_res.get_i32();
    let _transaction_id = connect_res.get_i32();
    let connection_id = connect_res.get_i64();

    let transaction_id = random::<i32>();

    let info_hash: [u8; 20] = meta_info.info_hash()?;

    let peer_id: [u8; 20] = random::<[u8; 20]>();

    let left = match meta_info.info {
        meta_info::Info::SingleFile { length, .. } => length,
        meta_info::Info::MultiFile { files, .. } => {
            files.iter().fold(0, |index, file| index + file.length)
        }
    };

    let key = random::<i32>();

    let mut announce_req = BytesMut::with_capacity(98);

    announce_req.put_i64(connection_id);
    announce_req.put_i32(ANNOUNCE);
    announce_req.put_i32(transaction_id);
    announce_req.put_slice(&info_hash);
    announce_req.put_slice(&peer_id);
    announce_req.put_i64(0); // downloaded
    announce_req.put_i64(left);
    announce_req.put_i64(0); // uploaded
    announce_req.put_i32(NONE);
    announce_req.put_i32(0); // ip address
    announce_req.put_i32(key);
    announce_req.put_i32(NUM_WANT);
    announce_req.put_i16(6881); // port should be between 6881 and 6889

    socket.send(&announce_req).await?;

    let mut announce_res = BytesMut::with_capacity(65508);
    announce_res.resize(65508, 0);

    let n = socket.recv(&mut announce_res).await?;
    announce_res.truncate(n);

    println!("Received {} bytes", n);

    let action = announce_res.get_i32();
    let transaction_id = announce_res.get_i32();
    let interval = announce_res.get_i32();
    let leechers = announce_res.get_i32();
    let seeders = announce_res.get_i32();

    println!("Remaining {}", announce_res.remaining());

    // let ip = announce_res.get_i32();
    // let port = announce_res.get_i16();

    println!(
        "action: {}\ntransaction_id: {}\ninterval: {}\nleechers: {}\nseeders: {}",
        action, transaction_id, interval, leechers, seeders
    );

    // println!("{}:{}", ip, port);

    Ok(())
}
