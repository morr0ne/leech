use anyhow::{anyhow, Result};
use bytes::{Buf, BufMut, BytesMut};
use rand::random;
use std::convert::TryFrom;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::{TcpStream, UdpSocket};

pub enum Actions {
    Connect = 0,
    Announce = 1,
    Scrape = 2,
}

impl TryFrom<i32> for Actions {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            x if x == Actions::Connect as i32 => Ok(Actions::Connect),
            x if x == Actions::Announce as i32 => Ok(Actions::Announce),
            x if x == Actions::Scrape as i32 => Ok(Actions::Scrape),
            _ => Err(anyhow!("")),
        }
    }
}

pub const NONE: i32 = 0;
pub const COMPLETED: i32 = 1;
pub const STARTED: i32 = 2;
pub const STOPPED: i32 = 3;

pub const NUM_WANT: i32 = -1;

pub const CHOKE: i32 = 0;
pub const UNCHOKE: i32 = 1;
pub const INTERESTED: i32 = 2;
pub const NOT_INTERESTED: i32 = 3;

pub struct Client {
    socket: UdpSocket,
}

impl Client {
    pub async fn new() -> Result<Client> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        Ok(Client { socket })
    }

    pub async fn connect(&self, url: &str) -> Result<ConnectResponse> {
        &self.socket.connect(url).await?;

        const PROTOCOL_ID: i64 = 0x41727101980;
        let transaction_id = random::<i32>();

        let mut connect_req = BytesMut::with_capacity(16);

        connect_req.put_i64(PROTOCOL_ID);
        connect_req.put_i32(Actions::Connect as i32);
        connect_req.put_i32(transaction_id);

        &self.socket.send(&connect_req).await?;

        let mut connect_res = BytesMut::with_capacity(16);
        connect_res.resize(16, 0);

        &self.socket.recv(&mut connect_res).await?;

        let action = Actions::try_from(connect_res.get_i32())?;
        let transaction_id = connect_res.get_i32();
        let connection_id = connect_res.get_i64();

        Ok(ConnectResponse {
            action,
            transaction_id,
            connection_id,
        })
    }

    pub async fn announce(
        self,
        connection_id: i64,
        info_hash: [u8; 20],
        left: i64,
    ) -> Result<AnnounceResponse> {
        let transaction_id = random::<i32>();

        let peer_id: [u8; 20] = random::<[u8; 20]>();

        let key = random::<i32>();
        let mut announce_req = BytesMut::with_capacity(98);

        announce_req.put_i64(connection_id);
        announce_req.put_i32(Actions::Announce as i32);
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

        self.socket.send(&announce_req).await?;

        let mut announce_res = BytesMut::with_capacity(65508);
        announce_res.resize(65508, 0);

        let n = self.socket.recv(&mut announce_res).await?;
        announce_res.truncate(n);

        let action = Actions::try_from(announce_res.get_i32())?;
        let transaction_id = announce_res.get_i32();
        let interval = announce_res.get_i32();
        let leechers = announce_res.get_i32();
        let seeders = announce_res.get_i32();
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
    pub transaction_id: i32,
    pub connection_id: i64,
}

pub struct AnnounceResponse {
    pub action: Actions,
    pub transaction_id: i32,
    pub interval: i32,
    pub leechers: i32,
    pub seeders: i32,
    pub peers: Vec<SocketAddrV4>,
}
