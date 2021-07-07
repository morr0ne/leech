use bytes::{BufMut, BytesMut};
use rand::random;

pub mod messages;

// TODO: There is definately a more efficient way to do this
pub fn peer_id(name: &[u8; 8]) -> [u8; 20] {
    let mut peer_id = BytesMut::with_capacity(20);
    peer_id.put(&name[..]);
    peer_id.put(&random::<[u8; 12]>()[..]);

    // SAFETY: This is safe because we know the lenght of bytes
    unsafe { slice_to_array(peer_id.as_ref()) }
}

// SAFETY: The caller must ensure the lenght fits
pub unsafe fn slice_to_array<T: Copy, const N: usize>(slice: &[T]) -> [T; N] {
    let ptr = slice.as_ptr() as *const [T; N];
    *ptr
}
