use bytes::{BufMut, BytesMut};
use rand::random;
use std::convert::TryInto;

// TODO: There is definately a more efficient way to do this
pub fn peer_id(name: &[u8; 8]) -> [u8; 20] {
    let mut peer_id = BytesMut::with_capacity(20);
    peer_id.put(&name[..]);
    peer_id.put(&random::<[u8; 12]>()[..]);

    match peer_id.as_ref().try_into() {
        Ok(peer_id) => return peer_id,
        Err(_) => unreachable!(), // This should never be reachable since we know the peer_id size already
    }
}
