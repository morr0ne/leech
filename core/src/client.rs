use anyhow::{bail, Result};
use bendy::decoding::{Error as DecodingError, FromBencode, ResultExt};
use bytes::{Buf, BufMut, BytesMut};
use form_urlencoded::byte_serialize;
use hyper::{body, client::HttpConnector, Body, Method, Request as HttpRequest, Uri};
use hyper_tls::HttpsConnector;
use rand::random;
use std::{
    convert::TryFrom,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
};
use tokio::net::UdpSocket;
use url::Url;

use crate::utils::ToArrayUnchecked;

pub type HttpClient<C = HttpsConnector<HttpConnector>> = hyper::Client<C>;

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
            _ => bail!("Unknown action"),
        }
    }
}

const NONE: u32 = 0;
// const COMPLETED: u32 = 1;
// const STARTED: u32 = 2;
// const STOPPED: u32 = 3;

pub struct Client {
    http_client: HttpClient,
    socket: UdpSocket,
}

impl Client {
    pub async fn new() -> Result<Client> {
        Ok(Client {
            http_client: HttpClient::builder().build(HttpsConnector::new()),
            socket: UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).await?,
        })
    }

    pub async fn announce(
        &self,
        url: &Url,
        announce_request: &AnnounceRequest,
    ) -> Result<Vec<SocketAddr>> {
        // self.announce_http(url.as_str(), announce_request).await?;
        let connect_response = self.connect_udp(url).await?;
        let announce_response = self
            .announce_udp(connect_response.connection_id, announce_request)
            .await?;

        Ok(announce_response.peers)
    }

    pub async fn announce_http(&self, url: &str, announce_request: &AnnounceRequest) -> Result<()> {
        let req = announce_request.into_http_request(&mut url.parse()?);

        let resp = self.http_client.request(req).await?.into_body();
        let body = body::to_bytes(resp).await?;

        println!("{:?}", &body);

        // let resp = HttpAnnounceResponse::from_bencode(&body)
        //     .expect("Failed to parse http announce response"); // TODO: better error handling

        Ok(())
    }

    pub async fn connect_udp(&self, url: &Url) -> Result<ConnectResponse> {
        // Build the tracker url using ip and port
        let url = format!("{}:{}", url.host_str().unwrap(), url.port().unwrap());

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

    pub async fn announce_udp(
        &self,
        connection_id: u64,
        announce_request: &AnnounceRequest,
    ) -> Result<AnnounceResponse> {
        let req = announce_request.into_udp_request(connection_id);
        self.socket.send(&req).await?;

        let mut announce_res = BytesMut::with_capacity(65508);
        announce_res.resize(65508, 0);

        let n = self.socket.recv(&mut announce_res).await?;
        announce_res.truncate(n);

        let action = Actions::try_from(announce_res.get_u32())?;
        let transaction_id = announce_res.get_u32();
        let interval = announce_res.get_u32();
        let leechers = announce_res.get_u32();
        let seeders = announce_res.get_u32();
        let mut peers: Vec<SocketAddr> = Vec::new();

        while 0 < announce_res.remaining() {
            let ip = Ipv4Addr::new(
                announce_res.get_u8(),
                announce_res.get_u8(),
                announce_res.get_u8(),
                announce_res.get_u8(),
            );
            peers.push(SocketAddr::new(IpAddr::V4(ip), announce_res.get_u16()))
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
    pub peers: Vec<SocketAddr>,
}

#[derive(Debug)]
pub struct AnnounceRequest {
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
    pub ip: Option<Ipv4Addr>,
    pub port: u16,
    pub uploaded: u64,
    pub downloaded: u64,
    pub left: u64,
}

#[derive(Debug)]
pub struct HttpAnnounceResponse {
    pub failure_reason: Option<String>,
    pub interval: Option<u64>,
    pub peers: Peer,
}

#[derive(Debug)]
pub struct Peer;

impl AnnounceRequest {
    pub fn into_http_request(&self, url: &mut Url) -> HttpRequest<Body> {
        // To send the info hash we first need to encode it in a http friendly format
        let mut encoded_info_hash = String::new();
        encoded_info_hash.extend(byte_serialize(&self.info_hash));

        // same as info hash
        let mut encoded_peer_id = String::new();
        encoded_peer_id.extend(byte_serialize(&self.peer_id));

        // After converting parsing the url we insert all the query parameters and convert it back to a string
        // The url standard only support utf-8 strings, we need to this so we can manually add the info hash and peer id later
        let mut url = url
            .query_pairs_mut()
            .append_pair("port", &self.port.to_string())
            .append_pair("uploaded", &self.uploaded.to_string())
            .append_pair("downloaded", &self.downloaded.to_string())
            .append_pair("left", &self.left.to_string())
            .finish()
            .to_string();

        // Manually add info hash and peer id
        url.push_str(&format!(
            "&info_hash={}&peer_id={}",
            encoded_info_hash, encoded_peer_id,
        ));

        let uri: Uri = url
            .parse()
            .expect("Internal error: A valid url should always resolve to a valid uri");

        HttpRequest::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Body::empty())
            .expect("")
    }

    pub fn into_udp_request(&self, connection_id: u64) -> [u8; 98] {
        let mut announce_req = Vec::with_capacity(98);

        let transaction_id = random::<u32>();
        let key = random::<u32>();

        announce_req.put_u64(connection_id);
        announce_req.put_u32(Actions::Announce as u32);
        announce_req.put_u32(transaction_id);
        announce_req.put_slice(&self.info_hash);
        announce_req.put_slice(&self.peer_id);
        announce_req.put_u64(self.downloaded);
        announce_req.put_u64(self.left);
        announce_req.put_u64(self.uploaded);
        announce_req.put_u32(NONE);
        announce_req.put_u32(0); // ip address
        announce_req.put_u32(key);
        announce_req.put_i32(-1); // num_want
        announce_req.put_u16(self.port);

        unsafe { announce_req.to_array_unchecked() }
    }
}

impl FromBencode for HttpAnnounceResponse {
    const EXPECTED_RECURSION_DEPTH: usize = 2048;

    fn decode_bencode_object(
        object: bendy::decoding::Object<'_, '_>,
    ) -> Result<Self, bendy::decoding::Error>
    where
        Self: Sized,
    {
        let mut dict_dec = object.try_into_dictionary()?;

        let mut failure_reason = None;
        let mut interval = None;
        let peers = None;

        while let Some((key, value)) = dict_dec.next_pair().context("pair")? {
            match key {
                b"failure reason" => {
                    failure_reason =
                        Some(String::decode_bencode_object(value).context("failure reason")?)
                }
                b"interval" => {
                    interval = Some(u64::decode_bencode_object(value).context("interval")?)
                }
                b"peers" => {}
                unknown_field => {
                    return Err(DecodingError::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )));
                }
            }
        }

        let peers = peers.unwrap();

        Ok(HttpAnnounceResponse {
            failure_reason,
            interval,
            peers,
        })
    }
}

// impl FromBencode for Peer {
//     const EXPECTED_RECURSION_DEPTH: usize = 2048;

//     fn decode_bencode_object(object: Object) -> Result<Self, DecodingError>
//     where
//         Self: Sized,
//     {
//         Ok(Peer {})
//     }
// }
