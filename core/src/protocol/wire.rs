use bitvec::{order::Msb0, slice::BitSlice};
use color_eyre::eyre::Result;
use futures::SinkExt;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, Error as TokioIoError};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

use super::{
    handshake::Handshake,
    message::{Message, MessageCodec},
};

#[derive(Debug, Error)]
pub enum HandshakeError {
    /// The remote peer returned an handshake of the correct size which couldn't be parsed.
    #[error("The peer returned an invalid handshake")]
    Invalid([u8; 68]),
    /// An error was encountered when trying to send the hanshake to the remote peer.
    #[error("Couldn't send handshake to peer")]
    Send(TokioIoError),
    /// An error was encountered when trying to read the handshake from the remote peer.
    #[error("Couldn't read handshake")]
    Read(TokioIoError),
    /// The remote peer returned an info hash different from the one sent
    #[error("Wrong info hash returned by peer")]
    InfoHash {
        expected: [u8; 20],
        received: [u8; 20],
    },
}

pub struct Wire<S> {
    stream: Framed<S, MessageCodec>,
}

#[derive(Debug)]
pub struct PeerInfo {
    pub peer_id: [u8; 20],
    pub extension_protocol: bool,
    pub fast_extension: bool,
    pub dht_extension: bool,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Wire<S> {
    pub async fn handshake(
        handshake: Handshake,
        mut stream: S,
    ) -> Result<(PeerInfo, Self), HandshakeError> {
        // As soon as we are connected send the handshake.
        stream
            .write_all(&handshake.as_bytes())
            .await
            .map_err(HandshakeError::Send)?;

        // Create a buffer for the handshake, fill it and then parse it.
        let mut remote_handshake_buffer = [0u8; 68];
        stream
            .read_exact(&mut remote_handshake_buffer)
            .await
            .map_err(HandshakeError::Read)?;
        let remote_handshake = Handshake::from_bytes(&remote_handshake_buffer)
            .map_err(|_error| HandshakeError::Invalid(remote_handshake_buffer))?;

        // Ensure the info hash matches.
        if handshake.info_hash != remote_handshake.info_hash {
            return Err(HandshakeError::InfoHash {
                expected: handshake.info_hash,
                received: handshake.info_hash,
            });
        }

        // Parse the reserved bytes as bit flags
        let reserved_bits: &BitSlice<u8, Msb0> =
            unsafe { BitSlice::from_slice_unchecked(&remote_handshake.reserved_bytes) };

        let extension_protocol = reserved_bits[43];
        let fast_extension = reserved_bits[61];
        let dht_extension = reserved_bits[63];

        Ok((
            PeerInfo {
                peer_id: remote_handshake.peer_id,
                extension_protocol,
                fast_extension,
                dht_extension,
            },
            Self {
                stream: Framed::new(stream, MessageCodec::new()),
            },
        ))
    }

    /// Read the next message in the stream
    pub async fn read_message(&mut self) -> Result<Option<Message>> {
        self.stream.try_next().await
    }

    /// Write a message to the internal sink, this does not flush
    pub async fn write_message(&mut self, message: Message) -> Result<()> {
        self.stream.feed(message).await
    }

    /// Flush all the pending messages
    pub async fn flush(&mut self) -> Result<()> {
        self.stream.flush().await
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use tokio_test::io::Builder as MockBuilder;

    #[tokio::test]
    async fn do_handshake() {
        // let stream = MockBuilder::new().build();
    }
}
