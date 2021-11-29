use anyhow::{anyhow, bail, Result};
use array_utils::ToArrayUnchecked;
use bento::{AsString, DecodingError, FromBencode};
use bytes::{Bytes, BytesMut};
use indexmap::IndexMap;
use nom::{
    bytes::complete::take, combinator::map_res, error::Error as NomError, multi::length_data,
    number::complete::be_u8, sequence::tuple, Finish,
};
use std::net::IpAddr;

pub struct Handshake {
    pub reserved_bytes: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub const fn new(reserved_bytes: [u8; 8], info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            reserved_bytes,
            info_hash,
            peer_id,
        }
    }

    pub fn from_bytes(bytes: &[u8; 68]) -> Result<Self> {
        map_res(
            tuple((
                length_data(be_u8),
                take(8usize),
                take(20usize),
                take(20usize),
            )),
            |(pstr, mut reserved_bytes, mut info_hash, mut peer_id): (
                &[u8],
                &[u8],
                &[u8],
                &[u8],
            )| {
                if pstr == b"BitTorrent protocol" {
                    Ok(Self {
                        reserved_bytes: unsafe { reserved_bytes.to_array_unchecked() },
                        info_hash: unsafe { info_hash.to_array_unchecked() },
                        peer_id: unsafe { peer_id.to_array_unchecked() },
                    })
                } else {
                    bail!("Invalid pstr")
                }
            },
        )(bytes)
        .finish()
        .map_err(|_error: NomError<&[u8]>| anyhow!("Failed to parse handshake"))
        .map(|(_rest, handshake)| handshake)
    }

    pub fn to_bytes(self) -> Bytes {
        let mut handshake = BytesMut::with_capacity(68);
        handshake.extend_from_slice(&[
            19, // pstrlen. Always 19 in the 1.0 protocol
            66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99, 111,
            108, // pstr. Always "BitTorrent protocol" in the 1.0 protocol
        ]);
        handshake.extend_from_slice(&self.reserved_bytes);
        handshake.extend_from_slice(&self.info_hash);
        handshake.extend_from_slice(&self.peer_id);
        handshake.freeze()
    }
}

#[derive(Debug)]
pub struct ExtendedHandshake {
    pub messages: IndexMap<String, u32>,
    pub port: Option<u16>,
    pub version: Option<String>,
    pub yourip: Option<IpAddr>,
    pub reqq: Option<u32>,
    pub metadata_size: Option<u32>,
}

impl FromBencode for ExtendedHandshake {
    fn decode(object: bento::Object<'_, '_>) -> Result<Self, bento::DecodingError>
    where
        Self: Sized,
    {
        let mut messages = None;
        let mut port = None;
        let mut version = None;
        let mut yourip = None;
        let mut reqq = None;
        let mut metadata_size = None;

        let mut dict_dec = object.try_dictionary()?;
        while let Some((key, value)) = dict_dec.next_pair()? {
            match key {
                b"m" => messages = value.decode()?,
                b"p" => port = value.decode()?,
                b"v" => version = value.decode()?,
                b"yourip" => yourip = Some(value.decode::<AsString<Vec<u8>>>()?.0),
                b"reqq" => reqq = value.decode()?,
                b"metadata_size" => metadata_size = value.decode()?,
                _unknown_field => value.skip()?,
            }
        }

        // TODO: should this be considered an error?
        let yourip = if let Some(mut ip) = yourip {
            if ip.len() == 4 {
                let ip: [u8; 4] = unsafe { ip.to_array_unchecked() };
                Some(IpAddr::from(ip))
            } else if ip.len() == 16 {
                let ip: [u8; 16] = unsafe { ip.to_array_unchecked() };
                Some(IpAddr::from(ip))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            messages: messages.ok_or(DecodingError::MissingField { field: "messages" })?,
            port,
            version,
            yourip,
            reqq,
            metadata_size,
        })
    }
}
