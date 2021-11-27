use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

use super::Message;

pub(crate) struct MessageCodec {
    inner: LengthDelimitedCodec,
}

impl MessageCodec {
    pub(crate) fn new() -> Self {
        Self {
            inner: LengthDelimitedCodec::builder().big_endian().new_codec(),
        }
    }
}

impl Decoder for MessageCodec {
    type Item = Message;

    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.inner.decode(src) {
            Ok(Some(bytes)) => Message::from_bytes(&bytes).map(Some),
            Ok(None) => Ok(None),
            Err(err) => Err(anyhow!(err)),
        }
    }
}

impl Encoder<Message> for MessageCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, message: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(message.len());
        message.extend_bytes(dst);
        Ok(())
    }
}
