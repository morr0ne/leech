use anyhow::Result;
use bencode::{decode::FromBencode, encode::ToBencode};
use metainfo::MetaInfo;
use std::fs;

fn main() -> Result<()> {
    let t = fs::read("temp/ubuntu.torrent")?;

    let meta_info = MetaInfo::from_bencode(&t).expect("Failed to parse torrent file");
    println!("{:#?}", meta_info);

    Ok(())
}
