use anyhow::Result;
use tokio::sync::{mpsc, oneshot};

use crate::Piece;

pub struct Session {
    peer_id: [u8; 20],
}

impl Session {
    pub fn builder() -> SessionBuilder {
        SessionBuilder::new()
    }

    pub async fn next_piece(&mut self) -> Result<Option<Piece>> {
        Ok(None)
    }
}

pub struct SessionBuilder {
    peer_id: [u8; 20],
}

impl SessionBuilder {
    pub fn new() -> Self {
        Self {
            peer_id: peers::peer_id(b"LE", b"0001"),
        }
    }

    pub async fn connect(&mut self) -> Result<Session> {
        Ok(Session {
            peer_id: self.peer_id,
        })
    }

    pub fn keep_alive_interval(&mut self) -> &mut Self {
        self
    }

    pub fn peer_id(&mut self, peer_id: [u8; 20]) -> &mut Self {
        self.peer_id = peer_id;
        self
    }
}

impl Default for SessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
