use anyhow::{anyhow, Result};
use array_utils::ToArrayUnchecked;
use bento::{DecodingError, FromBencode};
use bitvec::{order::Msb0, prelude::BitVec};
use bytes::{Bytes, BytesMut};
use indexmap::IndexMap;
use nom::{
    combinator::{map_res, rest},
    error::Error as NomError,
    number::complete::be_u8,
    sequence::pair,
    Finish,
};

mod codec;
mod handshake;
pub(crate) use codec::MessageCodec;
pub use handshake::Handshake;

#[derive(Debug)]
pub enum Message {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(BitVec<Msb0, u8>),
    Request {
        index: u32,
        begin: u32,
        length: u32,
    },
    Piece {
        index: u32,
        begin: u32,
        block: Bytes,
    },
    Cancel {
        index: u32,
        begin: u32,
        length: u32,
    },
    Port(u16),
    Extended {
        id: u8,
        payload: Bytes,
    },
    Unknown {
        id: u8,
        payload: Bytes,
    },
}

#[derive(Debug)]
pub struct ExtendedHandshake {
    pub messages: IndexMap<String, u32>,
    pub port: Option<u16>,
    pub version: Option<String>,
    pub yourip: Option<String>,
    pub reqq: Option<u32>,
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

        let mut dict_dec = object.try_dictionary()?;
        while let Some((key, value)) = dict_dec.next_pair()? {
            match key {
                b"m" => messages = Some(IndexMap::decode(value)?),
                b"p" => port = Some(u16::decode(value)?),
                b"v" => version = Some(String::decode(value)?),
                b"yourip" => yourip = Some(String::decode(value)?),
                b"reqq" => reqq = Some(u32::decode(value)?),
                _unknown_field => value.skip()?,
            }
        }

        Ok(Self {
            messages: messages.ok_or(DecodingError::MissingField { field: "messages" })?,
            port,
            version,
            yourip,
            reqq,
        })
    }
}

// TODO: This can be implemented using a macro which would also easily allow for more messages
impl Message {
    pub const fn keep_alive() -> Self {
        Self::KeepAlive
    }

    pub const fn choke() -> Self {
        Self::Choke
    }

    pub const fn unchoke() -> Self {
        Self::Unchoke
    }

    pub const fn interested() -> Self {
        Self::Interested
    }

    pub const fn not_interested() -> Self {
        Self::NotInterested
    }

    pub const fn have(piece_index: u32) -> Self {
        Self::Have(piece_index)
    }

    pub const fn bitfield(bitfield: BitVec<Msb0, u8>) -> Self {
        Self::Bitfield(bitfield)
    }

    pub const fn request(index: u32, begin: u32, length: u32) -> Self {
        Self::Request {
            index,
            begin,
            length,
        }
    }

    pub const fn piece(index: u32, begin: u32, block: Bytes) -> Self {
        Self::Piece {
            index,
            begin,
            block,
        }
    }

    pub const fn cancel(index: u32, begin: u32, length: u32) -> Self {
        Self::Cancel {
            index,
            begin,
            length,
        }
    }

    pub const fn port(port: u16) -> Self {
        Self::Port(port)
    }

    pub const fn uknown(id: u8, payload: Bytes) -> Self {
        Self::Unknown { id, payload }
    }
}

impl Message {
    // TODO: This function should be const but there are 2 blocker:
    // - Bytes.len() isn't const for some reason so we need to wait for https://github.com/tokio-rs/bytes/pull/516 to be merged
    // - BitVec.len() returns a the number of bits so we first need to call as_raw_slice which isn't const
    #[allow(clippy::len_without_is_empty)] // There's no reason to implement this cause the len can never be 0
    pub fn len(&self) -> usize {
        match self {
            Message::KeepAlive => 4,
            Message::Choke | Message::Unchoke | Message::Interested | Message::NotInterested => 5,
            Message::Have(_) => 9,
            Message::Bitfield(bitfield) => 5 + bitfield.as_raw_slice().len(),
            Message::Request { .. } | Message::Cancel { .. } => 17,
            Message::Piece { block, .. } => 9 + block.len(),
            Message::Port(_) => 7,
            Message::Extended { payload, .. } => payload.len() + 7,
            Message::Unknown { payload, .. } => payload.len() + 5,
        }
    }

    /// Extends the bytes with the message
    ///
    /// Note: This does not reserve the required space
    pub fn extend_bytes(self, bytes: &mut BytesMut) {
        match self {
            Message::KeepAlive => bytes.extend_from_slice(&[0, 0, 0, 0]),
            Message::Choke => bytes.extend_from_slice(&[0, 0, 0, 1, 0]),
            Message::Unchoke => bytes.extend_from_slice(&[0, 0, 0, 1, 1]),
            Message::Interested => bytes.extend_from_slice(&[0, 0, 0, 1, 2]),
            Message::NotInterested => bytes.extend_from_slice(&[0, 0, 0, 1, 3]),
            Message::Have(piece_index) => {
                bytes.extend_from_slice(&[0, 0, 0, 5, 4]);
                bytes.extend_from_slice(&piece_index.to_be_bytes());
            }
            Message::Bitfield(bitfield) => {
                let bitfield = bitfield.as_raw_slice();
                bytes.extend_from_slice(&(bitfield.len() as u32 + 1).to_be_bytes());
                bytes.extend_from_slice(&[5]);
                bytes.extend_from_slice(bitfield);
            }
            Message::Request {
                index,
                begin,
                length,
            } => {
                bytes.extend_from_slice(&[0, 0, 0, 13, 6]);
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes.extend_from_slice(&begin.to_be_bytes());
                bytes.extend_from_slice(&length.to_be_bytes());
            }
            Message::Piece {
                index,
                begin,
                block,
            } => {
                bytes.extend_from_slice(&(block.len() as u32 + 9).to_be_bytes());
                bytes.extend_from_slice(&[7]);
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes.extend_from_slice(&begin.to_be_bytes());
                bytes.extend_from_slice(&block);
            }
            Message::Cancel {
                index,
                begin,
                length,
            } => {
                bytes.extend_from_slice(&[0, 0, 0, 13, 8]);
                bytes.extend_from_slice(&index.to_be_bytes());
                bytes.extend_from_slice(&begin.to_be_bytes());
                bytes.extend_from_slice(&length.to_be_bytes());
            }
            Message::Port(port) => {
                bytes.extend_from_slice(&[0, 0, 0, 3, 9]);
                bytes.extend_from_slice(&port.to_be_bytes());
            }
            Message::Extended { id, payload } => {
                bytes.extend_from_slice(&(payload.len() as u32 + 2).to_be_bytes()); // Equal to the payload len + 1 for the id + 1 for the extended id
                bytes.extend_from_slice(&[20]); // Message id
                bytes.extend_from_slice(&[id]); // Extended message id
                bytes.extend_from_slice(&payload); // Payload
            }
            Message::Unknown { id, payload } => {
                bytes.extend_from_slice(&(payload.len() as u32 + 1).to_be_bytes()); // Equal to the payload len + 1 for the id
                bytes.extend_from_slice(&[id]); // Message id
                bytes.extend_from_slice(&payload); // Payload
            }
        }
    }

    pub fn to_bytes(self) -> Bytes {
        let mut message = BytesMut::with_capacity(self.len());
        self.extend_bytes(&mut message);
        message.freeze()
    }

    // TODO: refactor this function with a custom error type
    pub fn from_bytes(bytes: &[u8]) -> Result<Message> {
        if bytes.is_empty() {
            Ok(Message::KeepAlive)
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
                    5 => Ok(Message::Bitfield(BitVec::from_slice(payload).unwrap())), // TODO: handle errors
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
                                block: Bytes::copy_from_slice(&payload[8..]),
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
                    20 => {
                        let id = payload[0];
                        let payload = Bytes::copy_from_slice(&payload[1..]); // TODO: copy_from_slice is too expensive

                        Ok(Message::Extended { id, payload })
                    }
                    id => Ok(Message::Unknown {
                        id,
                        payload: Bytes::copy_from_slice(payload), // TODO: copy_from_slice is too expensive
                    }),
                },
            )(bytes)
            .finish()
            .map_err(|_err: NomError<&[u8]>| anyhow!(""))
            .map(|(_rest, message)| message)
        }
    }
}

impl TryFrom<&[u8]> for Message {
    type Error = anyhow::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::from_bytes(bytes)
    }
}

impl From<Message> for Bytes {
    fn from(message: Message) -> Self {
        message.to_bytes()
    }
}
