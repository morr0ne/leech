use anyhow::{bail, Result};
use bytes::{Buf, BufMut, BytesMut};
use hyper::{body, client::HttpConnector, Body, Method, Request as HttpRequest, Uri};
use hyper_tls::HttpsConnector;
use rand::random;
use std::{
    convert::TryFrom,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
};
use tokio::net::UdpSocket;
use tracker::tracker::http::{
    AnnounceRequest as HttpAnnounceRequest, AnnounceResponse as HttpAnnounceResponse, HttpTracker,
};
use url::Url;

pub type HttpClient<C = HttpsConnector<HttpConnector>> = hyper::Client<C>;

pub enum Actions {
    Connect = 0,
    Announce = 1,
    Scrape = 2,
    Error = 3,
}

impl TryFrom<u32> for Actions {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            0 => Ok(Actions::Connect),
            1 => Ok(Actions::Announce),
            2 => Ok(Actions::Scrape),
            3 => Ok(Actions::Error),
            x => bail!("Unknown action {}", x),
        }
    }
}

// const NONE: u32 = 0;
// const COMPLETED: u32 = 1;
// const STARTED: u32 = 2;
// const STOPPED: u32 = 3;

pub struct Client {
    http_tracker: HttpTracker,
    // socket: UdpSocket,
}

impl Client {
    pub async fn new() -> Result<Client> {
        Ok(Client {
            http_tracker: HttpTracker::new(),
            // socket: UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).await?,
        })
    }

    pub async fn announce(
        &self,
        url: &str,
        announce_request: &HttpAnnounceRequest,
    ) -> Result<HttpAnnounceResponse> {
        self.http_tracker.announce(url, announce_request).await
    }
}

impl Client {
    // pub async fn announce(
    //     &self,
    //     url: &Url,
    //     announce_request: &AnnounceRequest,
    // ) -> Result<Vec<SocketAddr>> {
    //     // self.announce_http(url.as_str(), announce_request).await?;
    //     let connect_response = self.connect_udp(url).await?;
    //     let announce_response = self
    //         .announce_udp(connect_response.connection_id, announce_request)
    //         .await?;

    //     Ok(announce_response.peers)
    // }

    // pub async fn connect_udp(&self, url: &Url) -> Result<ConnectResponse> {
    //     // Build the tracker url using ip and port
    //     let url = format!("{}:{}", url.host_str().unwrap(), url.port().unwrap());

    //     println!("Connecting to {}", &url);

    //     self.socket.connect(url).await?;

    //     const PROTOCOL_ID: u64 = 0x41727101980;
    //     let transaction_id = random::<u32>();

    //     let mut connect_req = BytesMut::with_capacity(16);

    //     connect_req.put_u64(PROTOCOL_ID);
    //     connect_req.put_u32(Actions::Connect as u32);
    //     connect_req.put_u32(transaction_id);

    //     self.socket.send(&connect_req).await?;

    //     let mut connect_res = BytesMut::with_capacity(16);
    //     connect_res.resize(16, 0);

    //     self.socket.recv(&mut connect_res).await?;

    //     let action = Actions::try_from(connect_res.get_u32())?;

    //     let transaction_id = connect_res.get_u32();

    //     if let Actions::Error = action {
    //         bail!("{}", String::from_utf8_lossy(&connect_res))
    //     }

    //     let connection_id = connect_res.get_u64();

    //     Ok(ConnectResponse {
    //         action,
    //         transaction_id,
    //         connection_id,
    //     })
    // }

    // pub async fn announce_udp(
    //     &self,
    //     connection_id: u64,
    //     announce_request: &AnnounceRequest,
    // ) -> Result<AnnounceResponse> {
    //     let req = announce_request.into_udp_request(connection_id);
    //     self.socket.send(&req).await?;

    //     let mut announce_res = BytesMut::with_capacity(65508);
    //     announce_res.resize(65508, 0);

    //     let n = self.socket.recv(&mut announce_res).await?;
    //     announce_res.truncate(n);

    //     let action = Actions::try_from(announce_res.get_u32())?;
    //     let transaction_id = announce_res.get_u32();
    //     if let Actions::Error = action {
    //         bail!("{}", String::from_utf8_lossy(&announce_res))
    //     }

    //     let interval = announce_res.get_u32();
    //     let leechers = announce_res.get_u32();
    //     let seeders = announce_res.get_u32();
    //     let mut peers: Vec<SocketAddr> = Vec::new();

    //     while 0 < announce_res.remaining() {
    //         let ip = Ipv4Addr::new(
    //             announce_res.get_u8(),
    //             announce_res.get_u8(),
    //             announce_res.get_u8(),
    //             announce_res.get_u8(),
    //         );
    //         peers.push(SocketAddr::new(IpAddr::V4(ip), announce_res.get_u16()))
    //     }

    //     Ok(AnnounceResponse {
    //         action,
    //         transaction_id,
    //         interval,
    //         leechers,
    //         seeders,
    //         peers,
    //     })
    // }
}

pub struct ConnectResponse {
    pub action: Actions,
    pub transaction_id: u32,
    pub connection_id: u64,
}
