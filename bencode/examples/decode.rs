use anyhow::Result;
use bencode::decode::decode;
use std::fs;

fn main() -> Result<()> {
    let file = fs::read("temp/bunny.torrent")?;
    let d = decode(&file).unwrap();

    // let decoder = Decoder::new(file);

    // let num = decode(b"i83472374e")?;
    // dbg!(num);

    //  letbyte_string = decode(b"8:federico")?;
    // dbg!(byte_string);

    // let list = decode(b"l4:spam4:eggse")?;
    // dbg!(list);

    Ok(())
}
