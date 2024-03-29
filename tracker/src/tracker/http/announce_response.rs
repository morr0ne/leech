use bento::{AsString, DecodingError, FromBencode, Object};
use color_eyre::eyre::{bail, eyre, Result};
use nom::{
    combinator::map,
    multi::many0,
    number::complete::{be_u128, be_u16, be_u32},
    sequence::tuple,
    Finish, IResult,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

#[derive(Debug)]
pub struct AnnounceResponse {
    pub interval: u64,
    pub peers: Vec<SocketAddr>,
}

#[derive(Debug)]
struct Peer(SocketAddr);

fn parse_peers(value: Object) -> Result<Vec<SocketAddr>> {
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

fn parse_compact_peers_v4<T: AsRef<[u8]>>(peers: T) -> Result<Vec<SocketAddr>> {
    let parsed_peers: IResult<&[u8], Vec<SocketAddr>> = map(
        many0(tuple((map(be_u32, Ipv4Addr::from), be_u16))),
        |addrs: Vec<(Ipv4Addr, u16)>| {
            addrs
                .into_iter()
                .map(|(ip, port)| SocketAddr::V4(SocketAddrV4::new(ip, port)))
                .collect()
        },
    )(peers.as_ref());

    let parsed_peers = parsed_peers
        .finish()
        .map_err(|_| eyre!("Couldn't parse compact peers v4"))?
        .1;

    Ok(parsed_peers)
}

fn parse_compact_peers_v6<T: AsRef<[u8]>>(peers: T) -> Result<Vec<SocketAddr>> {
    let parsed_peers: IResult<&[u8], Vec<SocketAddr>> = map(
        many0(tuple((map(be_u128, Ipv6Addr::from), be_u16))),
        |addrs: Vec<(Ipv6Addr, u16)>| {
            addrs
                .into_iter()
                .map(|(ip, port)| SocketAddr::V6(SocketAddrV6::new(ip, port, 0, 0)))
                .collect()
        },
    )(peers.as_ref());

    let parsed_peers = parsed_peers
        .finish()
        .map_err(|_| eyre!("Couldn't parse compact peers v6"))?
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
                b"interval" => interval = value.decode()?,
                b"peers" => peers.extend(parse_peers(value).unwrap()),
                b"peers6" => {
                    peers.extend(parse_compact_peers_v6(AsString::decode(value)?).unwrap())
                }
                _unknown_field => value.skip()?,
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
        let mut ip: Option<IpAddr> = None;
        let mut port = None;

        let mut dictionary_decoder = object.try_dictionary()?;

        while let Some((key, value)) = dictionary_decoder.next_pair()? {
            match key {
                b"ip" => ip = value.decode()?,
                b"port" => port = value.decode()?,
                _unknown_field => value.skip()?,
            }
        }

        Ok(Self(SocketAddr::from((
            ip.ok_or_else(|| DecodingError::missing_field("ip"))?,
            port.ok_or_else(|| DecodingError::missing_field("port"))?,
        ))))
    }
}
