use bytes::{BufMut, BytesMut};
use rand::random;

pub mod messages;

// Helper function to create a valid peer id
pub fn peer_id(name: &[u8; 8]) -> [u8; 20] {
    let mut peer_id = BytesMut::with_capacity(20);
    peer_id.put(&name[..]);
    peer_id.put(&random::<[u8; 12]>()[..]);

    // SAFETY: This is safe because we know the lenght of bytes
    unsafe { slice_to_array(peer_id) }
}

// SAFETY: The caller must ensure the lenght fits
pub unsafe fn slice_to_array<T, S, const N: usize>(slice: S) -> [T; N]
where
    T: Copy,
    S: AsRef<[T]>,
{
    let ptr = slice.as_ref().as_ptr() as *const [T; N];
    *ptr
}
