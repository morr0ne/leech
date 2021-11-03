use anyhow::{anyhow, bail, Result};
use array_utils::{build_array, ToArrayUnchecked};
use nom::{
    branch::alt,
    bytes::complete::take,
    combinator::{map_res, rest},
    multi::{length_data, length_value},
    number::complete::{be_u32, be_u8},
    sequence::{pair, tuple},
    Finish, IResult,
};

#[derive(Debug)]
pub enum Message {
    Handshake {
        reserved_bytes: [u8; 8],
        info_hash: [u8; 20],
        peer_id: [u8; 20],
    },
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

fn parse_message(bytes: &[u8]) -> IResult<&[u8], Message> {
    if bytes.is_empty() {
        Ok((bytes, Message::KeepAlive))
    } else {
        map_res(
            pair(be_u8, rest),
            |(id, mut payload): (u8, &[u8])| match id {
                0 => Ok(Message::Choke),
                1 => Ok(Message::Unchoke),
                2 => Ok(Message::Interested),
                3 => Ok(Message::NotInterested),
                4 => {
                    if let Ok(piece_index) = payload.try_into() {
                        Ok(Message::Have(u32::from_be_bytes(piece_index)))
                    } else {
                        Err(anyhow!("Invalid payload len"))
                    }
                }
                5 => Ok(Message::Bitfield), // TODO: parse bitfield
                6 => {
                    // TODO: is this the most efficient way to do this?
                    if payload.len() == 12 {
                        let payload: [u8; 12] = unsafe { payload.to_array_unchecked() };

                        Ok(Message::Request {
                            index: u32::from_be_bytes(unsafe {
                                (&payload[..4]).to_array_unchecked()
                            }),
                            begin: u32::from_be_bytes(unsafe {
                                (&payload[4..8]).to_array_unchecked()
                            }),
                            length: u32::from_be_bytes(unsafe {
                                (&payload[8..12]).to_array_unchecked()
                            }),
                        })
                    } else {
                        Err(anyhow!("Invalid payload len"))
                    }
                }
                7 => {
                    if payload.len() >= 8 {
                        Ok(Message::Piece {
                            index: u32::from_be_bytes(unsafe {
                                (&payload[..4]).to_array_unchecked()
                            }),
                            begin: u32::from_be_bytes(unsafe {
                                (&payload[4..8]).to_array_unchecked()
                            }),
                            block: payload[8..].to_vec(),
                        })
                    } else {
                        Err(anyhow!("Invalid payload len for Piece message"))
                    }
                }

                8 => {
                    // TODO: is this the most efficient way to do this?
                    if payload.len() == 12 {
                        let payload: [u8; 12] = unsafe { payload.to_array_unchecked() };

                        Ok(Message::Cancel {
                            index: u32::from_be_bytes(unsafe {
                                (&payload[..4]).to_array_unchecked()
                            }),
                            begin: u32::from_be_bytes(unsafe {
                                (&payload[4..8]).to_array_unchecked()
                            }),
                            length: u32::from_be_bytes(unsafe {
                                (&payload[8..12]).to_array_unchecked()
                            }),
                        })
                    } else {
                        Err(anyhow!("Invalid payload len for Cancel message"))
                    }
                }
                9 => {
                    if payload.len() == 2 {
                        Ok(Message::Port(u16::from_be_bytes(unsafe {
                            payload.to_array_unchecked()
                        })))
                    } else {
                        Err(anyhow!("Invalid payload len for Port message"))
                    }
                }
                _ => Err(anyhow!("Invalid message id {}", id)),
            },
        )(bytes)
    }
}

pub fn parse_handshake(bytes: &[u8]) -> IResult<&[u8], Message> {
    map_res(
        tuple((
            length_data(be_u8),
            take(8usize),
            take(20usize),
            take(20usize),
        )),
        |(pstr, mut reserved_bytes, mut info_hash, mut peer_id): (&[u8], &[u8], &[u8], &[u8])| {
            if pstr == b"BitTorrent protocol" {
                Ok(Message::Handshake {
                    reserved_bytes: unsafe { reserved_bytes.to_array_unchecked() },
                    info_hash: unsafe { info_hash.to_array_unchecked() },
                    peer_id: unsafe { peer_id.to_array_unchecked() },
                })
            } else {
                bail!("Invalid pstr")
            }
        },
    )(bytes)
}

impl Message {
    pub const KEEP_ALIVE: [u8; 4] = [0, 0, 0, 0];
    pub const CHOKE: [u8; 5] = [0, 0, 0, 1, 0];
    pub const UNCHOKE: [u8; 5] = [0, 0, 0, 1, 1];
    pub const INTERESTED: [u8; 5] = [0, 0, 0, 1, 2];
    pub const NOT_INTERESTED: [u8; 5] = [0, 0, 0, 1, 3];

    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = bytes.as_ref();

        let message: IResult<&[u8], Message> =
            alt((parse_handshake, length_value(be_u32, parse_message)))(bytes);
        let message = message
            .finish()
            .map_err(|_| anyhow!("Couldn't parse message"))?;

        Ok(message.1)
    }

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

impl TryFrom<&[u8]> for Message {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes)
    }
}
