use rand::random;

pub mod arrays;
pub mod messages;

pub use arrays::{build_array, ToArrayUnchecked};

/// Helper function to create a valid peer id
pub fn peer_id(name: &[u8; 8]) -> [u8; 20] {
    unsafe { build_array([name, &random::<[u8; 12]>()]) }
}
