use anyhow::{anyhow, bail, Result};
use array_utils::ToArrayUnchecked;
use nom::{
    bytes::complete::take, combinator::map_res, error::Error as NomError, multi::length_data,
    number::complete::be_u8, sequence::tuple, Finish,
};

pub struct Handshake {
    pub reserved_bytes: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
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
}
