use bytes::{BufMut, Bytes, BytesMut};
use rand::random;

pub fn peer_id(name: &[u8; 8]) -> Bytes {
    let mut peer_id = BytesMut::with_capacity(20);
    peer_id.put(&name[..]);
    peer_id.put(&random::<[u8; 12]>()[..]);

    peer_id.freeze()
}
