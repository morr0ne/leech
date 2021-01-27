use anyhow::{anyhow, Result};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use rand::random;
use std::convert::TryFrom;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::UdpSocket;

pub enum Actions {
    Connect = 0,
    Announce = 1,
    Scrape = 2,
}

impl TryFrom<u32> for Actions {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            x if x == Actions::Connect as u32 => Ok(Actions::Connect),
            x if x == Actions::Announce as u32 => Ok(Actions::Announce),
            x if x == Actions::Scrape as u32 => Ok(Actions::Scrape),
            _ => Err(anyhow!("Unknown action")),
        }
    }
}

const NONE: u32 = 0;
// const COMPLETED: u32 = 1;
// const STARTED: u32 = 2;
// const STOPPED: u32 = 3;
const NUM_WANT: i32 = -1;

pub struct Client {
    /// Technically the announce url could be using http but since almost all of them use udp for now I'll just use that
    socket: UdpSocket,
    /// Unique peer_id, in theory it can always be different
    pub peer_id: Bytes,
}

impl Client {
    pub async fn new(name: &[u8; 8]) -> Result<Client> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let mut peer_id = BytesMut::with_capacity(20);
        peer_id.put(&name[..]);
        peer_id.put(&random::<[u8; 12]>()[..]);

        Ok(Client {
            socket,
            peer_id: peer_id.freeze(),
        })
    }

    pub async fn connect(&self, url: &str) -> Result<ConnectResponse> {
        self.socket.connect(url).await?;

        const PROTOCOL_ID: u64 = 0x41727101980;
        let transaction_id = random::<u32>();

        let mut connect_req = BytesMut::with_capacity(16);

        connect_req.put_u64(PROTOCOL_ID);
        connect_req.put_u32(Actions::Connect as u32);
        connect_req.put_u32(transaction_id);

        self.socket.send(&connect_req).await?;

        let mut connect_res = BytesMut::with_capacity(16);
        connect_res.resize(16, 0);

        self.socket.recv(&mut connect_res).await?;

        let action = Actions::try_from(connect_res.get_u32())?;
        let transaction_id = connect_res.get_u32();
        let connection_id = connect_res.get_u64();

        Ok(ConnectResponse {
            action,
            transaction_id,
            connection_id,
        })
    }

    pub async fn announce(
        &self,
        connection_id: u64,
        info_hash: [u8; 20],
        left: u64,
    ) -> Result<AnnounceResponse> {
        let transaction_id = random::<u32>();

        let key = random::<u32>();
        let mut announce_req = BytesMut::with_capacity(98);

        announce_req.put_u64(connection_id);
        announce_req.put_u32(Actions::Announce as u32);
        announce_req.put_u32(transaction_id);
        announce_req.put_slice(&info_hash);
        announce_req.put_slice(&self.peer_id);
        announce_req.put_u64(0); // downloaded
        announce_req.put_u64(left);
        announce_req.put_u64(0); // uploaded
        announce_req.put_u32(NONE);
        announce_req.put_u32(0); // ip address
        announce_req.put_u32(key);
        announce_req.put_i32(NUM_WANT);
        announce_req.put_i16(6881); // port should be between 6881 and 6889

        self.socket.send(&announce_req).await?;

        let mut announce_res = BytesMut::with_capacity(65508);
        announce_res.resize(65508, 0);

        let n = self.socket.recv(&mut announce_res).await?;
        announce_res.truncate(n);

        let action = Actions::try_from(announce_res.get_u32())?;
        let transaction_id = announce_res.get_u32();
        let interval = announce_res.get_u32();
        let leechers = announce_res.get_u32();
        let seeders = announce_res.get_u32();
        let mut peers: Vec<SocketAddrV4> = Vec::new();

        while 0 < announce_res.remaining() {
            let ip = Ipv4Addr::new(
                announce_res.get_u8(),
                announce_res.get_u8(),
                announce_res.get_u8(),
                announce_res.get_u8(),
            );
            peers.push(SocketAddrV4::new(ip, announce_res.get_u16()))
        }

        Ok(AnnounceResponse {
            action,
            transaction_id,
            interval,
            leechers,
            seeders,
            peers,
        })
    }
}

pub struct ConnectResponse {
    pub action: Actions,
    pub transaction_id: u32,
    pub connection_id: u64,
}

pub struct AnnounceResponse {
    pub action: Actions,
    pub transaction_id: u32,
    pub interval: u32,
    pub leechers: u32,
    pub seeders: u32,
    pub peers: Vec<SocketAddrV4>,
}
