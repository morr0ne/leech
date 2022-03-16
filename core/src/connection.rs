use crate::{utp::UtpStream, Handshake, Status, Wire};
use color_eyre::eyre::Result;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpStream, ToSocketAddrs},
};

pub struct Connection<S> {
    status: Status,
    wire: Wire<S>,
    fast: bool,
}

pub struct ConnectionBuilder;

impl ConnectionBuilder {
    pub const fn new() -> Self {
        ConnectionBuilder {}
    }

    pub async fn connect_tcp<A: ToSocketAddrs>(
        addr: A,
        info_hash: [u8; 20],
        peer_id: [u8; 20],
    ) -> Result<Connection<TcpStream>> {
        let handshake = Handshake::new([0, 0, 0, 0, 0, 0x10, 0, 0], info_hash, peer_id);
        let stream = TcpStream::connect(addr).await?;
        let (peer_info, wire) = Wire::handshake(handshake, stream).await?;

        Ok(Connection {
            status: Status::new(),
            wire,
            fast: peer_info.fast_extension,
        })
    }

    pub async fn connect_utp<A: ToSocketAddrs>(
        addr: A,
        info_hash: [u8; 20],
        peer_id: [u8; 20],
    ) -> Result<Connection<UtpStream>> {
        let handshake = Handshake::new([0, 0, 0, 0, 0, 0x10, 0, 0], info_hash, peer_id);
        todo!()
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin> Connection<S> {}
