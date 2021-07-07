use bytes::{BufMut, BytesMut};

use super::slice_to_array;

pub const KEEP_ALIVE: [u8; 4] = [0, 0, 0, 0];
pub const CHOKE: [u8; 5] = [0, 0, 0, 0, 1];
pub const UNCHOKE: [u8; 5] = [0, 0, 0, 1, 1];
pub const INTERESTED: [u8; 5] = [0, 0, 0, 1, 2];
pub const NOT_INTERESTED: [u8; 5] = [0, 0, 0, 1, 3];

pub fn build_handshake(info_hash: &[u8; 20], peer_id: &[u8]) -> [u8; 68] {
    let mut handshake = BytesMut::with_capacity(68);

    handshake.put_slice(&[
        19, // pstrlen. Always 19 in the 1.0 protocol
        66, 105, 116, 84, 111, 114, 114, 101, 110, 116, 32, 112, 114, 111, 116, 111, 99, 111,
        108, // pstr. Always "BitTorrent protocol" in the 1.0 protocol
        0, 0, 0, 0, 0, 0, 0, 0, // reserved bytes. All current implementations use all zeroes
    ]);
    handshake.put_slice(info_hash); // torrent info hash
    handshake.put_slice(peer_id);

    // SAFETY: This is safe because we know the lenght of bytes
    unsafe { slice_to_array(handshake) }
}

pub fn build_have_message(piece_index: u32) -> [u8; 9] {
    let mut have = BytesMut::with_capacity(9);

    have.put_slice(&[
        0, 0, 0, 5, // len
        4, // id
    ]);
    have.put_u32(piece_index); // piece_index

    // SAFETY: This is safe because we know the lenght of bytes
    unsafe { slice_to_array(have) }
}

// I have yet to fully understand how this message works but since it's optional I'll look into it later
// fn build_bitfield_message(payload: BytesMut) -> BytesMut {
//     let mut bitfield = BytesMut::with_capacity(14);
//     bitfield.put_u32(u32::try_from(bitfield.len()).unwrap() + 1);
//     bitfield.p
// }

pub fn build_request_message(index: u32, begin: u32, length: u32) -> [u8; 17] {
    let mut request = BytesMut::with_capacity(17);

    request.put_slice(&[
        0, 0, 0, 13, // len
        6,  // id
    ]);
    request.put_u32(index);
    request.put_u32(begin);
    request.put_u32(length);

    // SAFETY: This is safe because we know the lenght of bytes
    unsafe { slice_to_array(request) }
}

pub fn build_cancel_message(index: u32, begin: u32, length: u32) -> [u8; 17] {
    let mut cancel = BytesMut::with_capacity(17);

    cancel.put_slice(&[
        0, 0, 0, 13, // len
        8,  // id
    ]);
    cancel.put_u32(index);
    cancel.put_u32(begin);
    cancel.put_u32(length);

    // SAFETY: This is safe because we know the lenght of bytes
    unsafe { slice_to_array(cancel) }
}

pub fn build_port_message(listen_port: u16) -> [u8; 7] {
    let mut port = BytesMut::with_capacity(7);

    port.put_slice(&[
        0, 0, 0, 3, // len
        9, // id
    ]);
    port.put_u16(listen_port);

    // SAFETY: This is safe because we know the lenght of bytes
    unsafe { slice_to_array(port) }
}
