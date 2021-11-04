use anyhow::{anyhow, bail, Result};
use array_utils::{build_array, ToArrayUnchecked};
use nom::{
    bytes::complete::take, combinator::map_res, multi::length_data, number::complete::be_u8,
    sequence::tuple, Finish, IResult,
};

pub struct Handshake {
    pub reserved_bytes: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn from_bytes(bytes: &[u8; 68]) -> Result<Self> {
        let handshake: IResult<&[u8], Self> = map_res(
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
        )(bytes);

        handshake
            .finish()
            .map_err(|_| anyhow!("Failed to parse handshake"))
            .map(|(_, handshake)| handshake)
    }
}

#[derive(Debug)]
pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield,
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece {
        index: u32,
        begin: u32,
        block: Vec<u8>, // TODO: should this be a byte slice?
    },
    Cancel {
        index: u32,
        begin: u32,
        length: u32,
    },
    Port(u16),
}

impl Message {
    pub const KEEP_ALIVE: [u8; 4] = [0, 0, 0, 0];
    pub const CHOKE: [u8; 5] = [0, 0, 0, 1, 0];
    pub const UNCHOKE: [u8; 5] = [0, 0, 0, 1, 1];
    pub const INTERESTED: [u8; 5] = [0, 0, 0, 1, 2];
    pub const NOT_INTERESTED: [u8; 5] = [0, 0, 0, 1, 3];

    pub fn handshake(reserved_bytes: [u8; 8], info_hash: [u8; 20], peer_id: [u8; 20]) -> [u8; 68] {
        unsafe {
            build_array([
                &[
                    19, // pstrlen. Always 19 in the 1.0 protocol
                    66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111,
                    99, 111, 108, // pstr. Always "BitTorrent protocol" in the 1.0 protocol
                ],
                &reserved_bytes,
                &info_hash,
                &peer_id,
            ])
        }
    }

    pub fn have(piece_index: u32) -> [u8; 9] {
        unsafe {
            build_array([
                &[
                    0, 0, 0, 5, // len
                    4, // id
                ],
                &piece_index.to_be_bytes(),
            ])
        }
    }

    // I have yet to fully understand how this message works but since it's optional I'll look into it later
    fn bitfield(payload: &[u8]) -> &[u8] {
        todo!()
    }

    pub fn request(index: u32, begin: u32, length: u32) -> [u8; 17] {
        unsafe {
            build_array([
                &[
                    0, 0, 0, 13, // len
                    6,  // id
                ],
                &index.to_be_bytes(),
                &begin.to_be_bytes(),
                &length.to_be_bytes(),
            ])
        }
    }

    pub fn piece<const N: usize>(index: u32, begin: u32, block: [u8; N]) -> [u8; N + 9] {
        unsafe {
            build_array([
                &(9 + N).to_be_bytes(),
                &[8],
                &index.to_be_bytes(),
                &begin.to_be_bytes(),
                &block,
            ])
        }
    }

    pub fn cancel(index: u32, begin: u32, length: u32) -> [u8; 17] {
        unsafe {
            build_array([
                &[
                    0, 0, 0, 13, // len
                    8,  // id
                ],
                &index.to_be_bytes(),
                &begin.to_be_bytes(),
                &length.to_be_bytes(),
            ])
        }
    }

    pub fn port(listen_port: u16) -> [u8; 7] {
        unsafe {
            build_array([
                &[
                    0, 0, 0, 3, // len
                    9, // id
                ],
                &listen_port.to_be_bytes(),
            ])
        }
    }
}
