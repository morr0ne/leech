use anyhow::{bail, Result};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader, Error as TokioIoError},
    net::{tcp::OwnedWriteHalf, TcpStream, ToSocketAddrs},
    sync::mpsc::{self, UnboundedReceiver},
};

use bitvec::prelude::*;

use crate::message::{Handshake, Message};

pub struct Status {
    /// Are we are choking the remote peer?
    pub am_choking: bool,
    /// Are we are interested the remote peer?
    pub am_interested: bool,
    /// Is the remote peer choking us?
    pub peer_choking: bool,
    /// Is the remote peer interested in us?
    pub peer_interested: bool,
}

impl Status {
    pub fn new() -> Self {
        Self {
            am_choking: true,
            am_interested: false,
            peer_choking: true,
            peer_interested: false,
        }
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum WireError {
    #[error("")]
    IoError(#[from] TokioIoError),
    #[error("Unknown error")]
    Unknown(#[from] anyhow::Error),
}

/// Error returned when failing to connect to a remote peer
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("The peer returned an invalid handshake")]
    InvalidHandshake([u8; 68]),
    #[error("Couldn't send handshake to peer")]
    Handshake(TokioIoError),
    #[error("Couldn't read handshake")]
    ReadHandshake(TokioIoError),
    #[error("Couldn't connect to remote peer")]
    TcpConnection(TokioIoError),
    /// The peer returned
    #[error("Wrong info hash returned by peer")]
    WrongInfoHash {
        expected: [u8; 20],
        received: [u8; 20],
    },
}

pub struct Wire {
    status: Status,
    pub message_receiver: UnboundedReceiver<Result<Message>>,
    write_socket: OwnedWriteHalf,
}

impl Wire {
    /// Connects to a remote peer
    pub async fn connect<A: ToSocketAddrs>(
        peer_address: A,
        info_hash: [u8; 20],
        peer_id: [u8; 20],
    ) -> Result<(Self, [u8; 20]), ConnectionError> {
        // Connect to the peer addres.
        let mut socket = TcpStream::connect(peer_address)
            .await
            .map_err(ConnectionError::TcpConnection)?;

        // As soon as we are connected send the handshake.
        socket
            .write_all(&Message::handshake([0u8; 8], info_hash, peer_id))
            .await
            .map_err(ConnectionError::Handshake)?;

        // Create a buffer for the handshake, fill it and then parse it.
        let mut handshake_buffer = [0u8; 68];
        socket
            .read_exact(&mut handshake_buffer)
            .await
            .map_err(ConnectionError::ReadHandshake)?;
        let handshake = Handshake::from_bytes(&handshake_buffer)
            .map_err(|_error| ConnectionError::InvalidHandshake(handshake_buffer))?;

        // Ensure the info hash matches.
        if info_hash != handshake.info_hash {
            return Err(ConnectionError::WrongInfoHash {
                expected: info_hash,
                received: handshake.info_hash,
            });
        }

        // Split the socket into read and writes halves to that they can be sent to separate tasks.
        let (read_socket, write_socket) = socket.into_split();

        // Created a mpsc channel so we can send Messages back.
        let (message_sender, message_receiver) = mpsc::unbounded_channel::<Result<Message>>();

        // Spawn a task that listen for the remote peer messages
        tokio::spawn(async move {
            let mut buffered_read_socket = BufReader::new(read_socket);
            loop {
                let message = Self::read_message(&mut buffered_read_socket).await.unwrap();
                // message_sender.send(message).unwrap();

                match message {
                    Message::Unchoke => {
                        println!("Peer stopped choking")
                    }
                    Message::Unknown { id, payload } => {
                        println!("Uknown message id {}", id)
                    }
                    Message::Bitfield(bitfield) => {
                        println!("Peer sent bitfield")
                    }
                    _ => {
                        dbg!(message);
                    }
                };
            }
        });

        Ok((
            Self {
                status: Status::new(),
                message_receiver,
                write_socket,
            },
            handshake.peer_id,
        ))
    }

    pub async fn read_message<A: AsyncRead + Unpin>(reader: &mut A) -> Result<Message> {
        let len = reader.read_u32().await?;
        Ok(if len == 0 {
            Message::KeepAlive
        } else {
            let id = reader.read_u8().await.unwrap();
            match id {
                0 => Message::Choke,
                1 => Message::Unchoke,
                2 => Message::Interested,
                3 => Message::NotInterested,
                4 => Message::Have(reader.read_u32().await?),
                5 => {
                    if len >= 1 {
                        let mut bitfield = vec![0u8; len as usize];
                        reader.read_exact(&mut bitfield).await?;

                        Message::Bitfield(BitVec::from_vec(bitfield)) // TODO: This can panic, the error should be handled instead with try_from_vec
                    } else {
                        bail!("Invalid payload len for Piece message")
                    }
                }
                6 => {
                    if len == 12 {
                        Message::Request {
                            index: reader.read_u32().await?,
                            begin: reader.read_u32().await?,
                            length: reader.read_u32().await?,
                        }
                    } else {
                        bail!("Invalid payload len")
                    }
                }
                7 => {
                    if len >= 8 {
                        let mut block = vec![0u8; len as usize];
                        reader.read_exact(&mut block).await?;

                        Message::Piece {
                            index: reader.read_u32().await?,
                            begin: reader.read_u32().await?,
                            block,
                        }
                    } else {
                        bail!("Invalid payload len for Piece message")
                    }
                }
                8 => {
                    if len == 12 {
                        Message::Cancel {
                            index: reader.read_u32().await?,
                            begin: reader.read_u32().await?,
                            length: reader.read_u32().await?,
                        }
                    } else {
                        bail!("Invalid payload len for Cancel message")
                    }
                }
                9 => {
                    if len == 2 {
                        Message::Port(reader.read_u16().await?)
                    } else {
                        bail!("Invalid payload len for Port message")
                    }
                }
                id => {
                    let mut payload = vec![0u8; len as usize];
                    reader.read_exact(&mut payload).await?;
                    Message::Unknown { id, payload }
                }
            }
        })
    }

    pub async fn keep_alive(&mut self) -> Result<()> {
        // Keep alives don't do anything special so we simply send the message
        self.write_socket.write_all(&Message::KEEP_ALIVE).await?;
        Ok(())
    }

    pub async fn choke(&mut self) -> Result<()> {
        // If we are not already choking send the choke message and update the status
        // TODO: stop outgoing connections
        if !self.status.am_choking {
            self.write_socket.write_all(&Message::CHOKE).await?;
            self.status.am_choking = true;
        }
        Ok(())
    }

    pub async fn unchoke(&mut self) -> Result<()> {
        // If we are choking send the unchoke message and update the status
        if self.status.am_choking {
            self.write_socket.write_all(&Message::UNCHOKE).await?;
            self.status.am_choking = false;
        }
        Ok(())
    }

    pub async fn interested(&mut self) -> Result<()> {
        // Unless we are already interested send the interested message and update the status
        if !self.status.am_interested {
            self.write_socket.write_all(&Message::INTERESTED).await?;
            self.status.am_interested = true;
        }
        Ok(())
    }

    pub async fn not_interested(&mut self) -> Result<()> {
        // If we are interested send the interested message and update the status
        if self.status.am_interested {
            self.write_socket
                .write_all(&Message::NOT_INTERESTED)
                .await?;
            self.status.am_interested = false;
        }
        Ok(())
    }

    pub async fn have(&mut self, piece_index: u32) -> Result<()> {
        self.write_socket
            .write_all(&Message::have(piece_index))
            .await?;
        Ok(())
    }

    pub async fn bitfield(&mut self) -> Result<()> {
        todo!()
    }
}
