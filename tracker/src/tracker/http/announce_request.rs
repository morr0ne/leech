use form_urlencoded::byte_serialize;
use hyper::{Body, Method, Request as HttpRequest, Uri};
use std::net::Ipv4Addr;
use url::Url;

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
    pub numwant: Option<u16>,
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
        let mut url_query = url.query_pairs_mut();
        url_query
            .append_pair("port", &self.port.to_string())
            .append_pair("uploaded", &self.uploaded.to_string())
            .append_pair("downloaded", &self.downloaded.to_string())
            .append_pair("left", &self.left.to_string())
            .append_pair("compact", &(self.compact as u8).to_string());

        if let Some(numwant) = self.numwant {
            url_query.append_pair("numwant", &numwant.to_string());
        }

        let mut url = url_query.finish().to_string();

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
