use anyhow::Result;
use form_urlencoded::byte_serialize;
use hyper::{client::HttpConnector, Body, Method, Request as HttpRequest, Uri};
use hyper_tls::HttpsConnector;
use std::net::{Ipv4Addr, SocketAddr};
use url::Url;

pub type HttpClient<C = HttpsConnector<HttpConnector>> = hyper::Client<C>;

pub struct HttpTracker {
    http_client: HttpClient,
}
