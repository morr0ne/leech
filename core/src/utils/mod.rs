use bytes::{BufMut, BytesMut};
use rand::random;

pub mod messages;

/// Helper function to create a valid peer id
pub fn peer_id(name: &[u8; 8]) -> [u8; 20] {
    unsafe { build_array([name, &random::<[u8; 12]>()]) }
}

/// SAFETY: The caller must ensure the lenght fits
#[inline]
pub unsafe fn slice_to_array<T, S, const N: usize>(slice: S) -> [T; N]
where
    T: Copy,
    S: AsRef<[T]>,
{
    let ptr = slice.as_ref().as_ptr() as *const [T; N];
    *ptr
}

/// Builds an array of N size from bytes.
///
/// SAFETY: The caller must payload bytes must be exactly of lenght N
pub unsafe fn build_array<const P: usize, const N: usize>(payload: [&[u8]; P]) -> [u8; N] {
    let mut message = BytesMut::with_capacity(N);

    for p in payload {
        message.put(p)
    }

    slice_to_array(message)
}
