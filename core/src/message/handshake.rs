use anyhow::{bail, Result};
use array_utils::{build_array, ToArrayUnchecked};
use nom::{
    bytes::complete::take, multi::length_data, number::complete::be_u8, sequence::tuple, IResult,
};

pub struct Handshake {
    pub reserved_bytes: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    // TODO: proper error handling
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = bytes.as_ref();
        if bytes.len() < 68 {
            bail!("Handshake too short")
        } else {
            let handshake: IResult<&[u8], (&[u8], &[u8], &[u8], &[u8])> = tuple((
                length_data(be_u8),
                take(8usize),
                take(20usize),
                take(20usize),
            ))(bytes);

            if let Ok((_rem, (pstr, mut reserved_bytes, mut info_hash, mut peer_id))) = handshake {
                if pstr == b"BitTorrent protocol" {
                    Ok(Self {
                        reserved_bytes: unsafe { reserved_bytes.to_array_unchecked() },
                        info_hash: unsafe { info_hash.to_array_unchecked() },
                        peer_id: unsafe { peer_id.to_array_unchecked() },
                    })
                } else {
                    bail!("Invalid pstr")
                }
            } else {
                bail!("Couldn't parse handshake")
            }
        }
    }

    pub fn into_bytes(&self) -> [u8; 68] {
        unsafe {
            build_array([
                &[
                    19, // pstrlen. Always 19 in the 1.0 protocol
                    66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111,
                    99, 111, 108, // pstr. Always "BitTorrent protocol" in the 1.0 protocol
                ],
                &self.reserved_bytes,
                &self.info_hash,
                &self.peer_id,
            ])
        }
    }
}

impl From<Handshake> for [u8; 68] {
    fn from(handshake: Handshake) -> Self {
        handshake.into_bytes()
    }
}

impl TryFrom<&[u8]> for Handshake {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Handshake::from_bytes(bytes)
    }
}
