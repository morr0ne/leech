use color_eyre::eyre::Result;
use bento::FromBencode;
use hyper::{body, client::HttpConnector};
use hyper_tls::HttpsConnector;

pub type HttpClient<C = HttpsConnector<HttpConnector>> = hyper::Client<C>;

mod announce_request;
mod announce_response;

pub use announce_request::{AnnounceRequest, Event};
pub use announce_response::AnnounceResponse;

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
