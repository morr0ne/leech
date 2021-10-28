use anyhow::{anyhow, bail, Result};
use bento::{
    decode::{DecodingError, FromBencode, Object},
    AsString,
};
use form_urlencoded::byte_serialize;
use hyper::{body, client::HttpConnector, Body, Method, Request as HttpRequest, Uri};
use hyper_tls::HttpsConnector;
use nom::{combinator::map, multi::many0, number::Endianness, sequence::tuple, Finish, IResult};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use url::Url;

pub type HttpClient<C = HttpsConnector<HttpConnector>> = hyper::Client<C>;

#[derive(Debug)]
pub struct AnnounceRequest {
    /// The 20 byte sha1 hash of the bencoded form of the info value from the metainfo file.
    pub info_hash: [u8; 20],
    /// A string which this downloader uses as its id.
    /// Each downloader generates its own id at random at the start of a new download.
    pub peer_id: [u8; 20],
    /// An optional parameter giving the IP (or dns name) which this peer is at.
    /// Generally used for the origin if it's on the same machine as the tracker.
    pub ip: Option<Ipv4Addr>,
    /// The port number this peer is listening on.
    /// Common behavior is for a downloader to try to listen on port 6881 and if that port is taken try 6882, then 6883, etc. and give up after 6889.
    pub port: u16,
    /// The total amount uploaded so far, encoded in base ten ascii.
    pub uploaded: u64,
    /// The total amount downloaded so far, encoded in base ten ascii.
    pub downloaded: u64,
    /// The number of bytes this peer still has to download, encoded in base ten ascii.
    /// Note that this can't be computed from downloaded and the file length since it might be a resume, and there's a chance that some of the downloaded data failed an integrity check and had to be re-downloaded.
    pub left: u64,
    /// If not present, this is one of the announcements done at regular intervals.
    /// An announcement using started is sent when a download first begins, and one using completed is sent when the download is complete.
    /// No completed is sent if the file was complete when started.
    /// Downloaders send an announcement using stopped when they cease downloading.
    pub event: Option<Event>,
    pub compact: bool,
}

#[derive(Debug)]
pub enum Event {
    Started,
    Completed,
    Stopped,
    Empty, // Same as None
}

impl AnnounceRequest {
    pub fn into_http_request(&self, url: &mut Url) -> HttpRequest<Body> {
        // To send the info hash we first need to encode it in a http friendly format
        let mut encoded_info_hash = String::new();
        encoded_info_hash.extend(byte_serialize(&self.info_hash));

        // same as info hash
        let mut encoded_peer_id = String::new();
        encoded_peer_id.extend(byte_serialize(&self.peer_id));

        // After converting parsing the url we insert all the query parameters and convert it back to a string
        // The url standard only support utf-8 strings, we need to do this so we can manually add the info hash and peer id later
        let mut url = url
            .query_pairs_mut()
            .append_pair("port", &self.port.to_string())
            .append_pair("uploaded", &self.uploaded.to_string())
            .append_pair("downloaded", &self.downloaded.to_string())
            .append_pair("left", &self.left.to_string())
            .append_pair("compact", &(self.compact as u8).to_string())
            .finish()
            .to_string();

        // Manually add info hash and peer id
        url.push_str(&format!(
            "&info_hash={}&peer_id={}",
            encoded_info_hash, encoded_peer_id,
        ));

        // Hyper doesn't accept the str directly but unless there's something wrong with the code above this should never panic
        let uri: Uri = url
            .parse()
            .expect("Internal error: A valid url should always resolve to a valid uri");

        HttpRequest::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Body::empty())
            .expect("")
    }
}

#[derive(Debug)]
pub struct AnnounceResponse {
    pub interval: u64,
    pub peers: Vec<SocketAddr>,
}

#[derive(Debug)]
pub struct Peer(SocketAddr);

pub fn parse_peers(value: Object) -> Result<Vec<SocketAddr>> {
    if value.is_list() {
        Ok(Vec::<Peer>::decode(value)?
            .into_iter()
            .map(|peer| peer.0)
            .collect())
    } else if value.is_byte_string() {
        parse_compact_peers_v4(unsafe { value.byte_string().unwrap_unchecked() })
    } else {
        bail!("")
    }
}

pub fn parse_compact_peers_v4<T: AsRef<[u8]>>(peers: T) -> Result<Vec<SocketAddr>> {
    use nom::number::complete::{u16, u32};

    let parsed_peers: IResult<&[u8], Vec<SocketAddr>> = map(
        many0(tuple((
            map(u32(Endianness::Big), Ipv4Addr::from),
            u16(Endianness::Big),
        ))),
        |addrs: Vec<(Ipv4Addr, u16)>| {
            addrs
                .into_iter()
                .map(|(ip, port)| SocketAddr::V4(SocketAddrV4::new(ip, port)))
                .collect()
        },
    )(peers.as_ref());

    let parsed_peers = parsed_peers
        .finish()
        .map_err(|_| anyhow!("Couldn't parse compact peers v4"))?
        .1;

    Ok(parsed_peers)
}

pub fn parse_compact_peers_v6<T: AsRef<[u8]>>(peers: T) -> Result<Vec<SocketAddr>> {
    use nom::number::complete::{u128, u16};

    let parsed_peers: IResult<&[u8], Vec<SocketAddr>> = map(
        many0(tuple((
            map(u128(Endianness::Big), Ipv6Addr::from),
            u16(Endianness::Big),
        ))),
        |addrs: Vec<(Ipv6Addr, u16)>| {
            addrs
                .into_iter()
                .map(|(ip, port)| SocketAddr::V6(SocketAddrV6::new(ip, port, 0, 0)))
                .collect()
        },
    )(peers.as_ref());

    let parsed_peers = parsed_peers
        .finish()
        .map_err(|_| anyhow!("Couldn't parse compact peers v6"))?
        .1;

    Ok(parsed_peers)
}

impl FromBencode for AnnounceResponse {
    fn decode(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut interval = None;
        let mut peers = Vec::new();

        let mut dictionary_decoder = object.try_dictionary()?;
        while let Some((key, value)) = dictionary_decoder.next_pair()? {
            match key {
                b"interval" => interval = Some(u64::decode(value)?),
                b"peers" => peers.extend(parse_peers(value).unwrap()),
                b"peers6" => {
                    peers.extend(parse_compact_peers_v6(AsString::decode(value)?).unwrap())
                }
                _ => {}
            }
        }

        Ok(Self {
            interval: interval.ok_or_else(|| DecodingError::missing_field("interval"))?,
            peers,
        })
    }
}

impl FromBencode for Peer {
    fn decode(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut ip = None;
        let mut port = None;

        let mut dictionary_decoder = object.try_dictionary()?;

        while let Some((key, value)) = dictionary_decoder.next_pair()? {
            match key {
                b"ip" => ip = Some(IpAddr::decode(value)?),
                b"port" => port = Some(u16::decode(value)?),
                _ => {}
            }
        }

        Ok(Self(SocketAddr::from((
            ip.ok_or_else(|| DecodingError::missing_field("ip"))?,
            port.ok_or_else(|| DecodingError::missing_field("port"))?,
        ))))
    }
}

pub struct HttpTracker {
    http_client: HttpClient,
}

impl HttpTracker {
    pub fn new() -> Self {
        Self {
            http_client: HttpClient::builder().build(HttpsConnector::new()),
        }
    }

    pub async fn announce(
        &self,
        url: &str,
        announce_request: &AnnounceRequest,
    ) -> Result<AnnounceResponse> {
        let req = announce_request.into_http_request(&mut url.parse()?);
        let resp = self.http_client.request(req).await?.into_body();
        let body = body::to_bytes(resp).await?;

        Ok(AnnounceResponse::from_bencode(&body)?)
    }
}

impl Default for HttpTracker {
    fn default() -> Self {
        Self::new()
    }
}
