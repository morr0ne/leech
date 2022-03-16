use color_eyre::eyre::Result;
use form_urlencoded::byte_serialize;
use hyper::{client::HttpConnector, Body, Method, Request as HttpRequest, Uri};
use hyper_tls::HttpsConnector;
use std::net::{Ipv4Addr, SocketAddr};
use url::Url;

pub type HttpClient<C = HttpsConnector<HttpConnector>> = hyper::Client<C>;

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
pub struct AnnounceResponse {
    pub action: u32,
    pub transaction_id: u32,
    pub interval: u32,
    pub leechers: u32,
    pub seeders: u32,
    pub peers: Vec<SocketAddr>,
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
}

pub struct Tracker {
    http_client: HttpClient,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    Ok(())
}
