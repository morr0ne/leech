// pub mod metainfo;

pub struct Torrent {
    info_hash: InfoHash,
}

pub enum InfoHash {
    V1([u8; 20]),
    V2,
}
