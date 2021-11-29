use anyhow::Result;
use futures::Stream;

pub enum Event {}

pub struct Session {
    peer_id: [u8; 20],
    // connections: Vec<Wire>,
}

impl Session {
    pub fn builder() -> SessionBuilder {
        SessionBuilder::new()
    }

    pub fn add_torrent(&mut self) {}
}

impl Stream for Session {
    type Item = Result<Event>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        todo!()
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

    pub async fn start(self) -> Session {
        Session {
            peer_id: self.peer_id,
            // connections: Vec::new(),
        }
    }

    pub fn announce_timeout(&mut self) -> &mut Self {
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
