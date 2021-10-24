use anyhow::Result;
use bencode::decode::FromBencode;
use metainfo::MetaInfo;
use std::fs;

fn main() -> Result<()> {
    let t = fs::read("temp/debian.torrent")?;

    let meta_info = MetaInfo::from_bencode(&t).expect("Failed to parse torrent file");
    println!("{:#?}", meta_info);

    Ok(())
}
