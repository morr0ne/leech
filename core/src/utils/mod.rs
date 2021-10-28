use rand::random;

pub mod messages;

/// Helper function to create a valid peer id
pub fn peer_id(name: &[u8; 8]) -> [u8; 20] {
    unsafe { array_utils::build_array([name, &random::<[u8; 12]>()]) }
}
