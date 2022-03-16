use color_eyre::eyre::{bail, Result};
use data_encoding::BASE32;
use std::convert::TryInto;
use url::Url;

#[derive(Debug)]
pub struct Magnet {
    pub info_hash: [u8; 20],
    pub display_name: Option<String>,
    pub trackers: Vec<String>,
    pub peers: Vec<String>,
}

pub fn parse(url: &str) -> Result<Magnet> {
    let url: Url = url.parse()?;

    let mut xt = None;
    let mut display_name = None;
    let mut trackers = Vec::new();
    let mut peers = Vec::new();

    for (key, value) in url.query_pairs().into_owned() {
        match key.as_str() {
            "xt" => xt = Some(value),
            "dn" => display_name = Some(value),
            "tr" => trackers.push(value),
            "x.pe" => peers.push(value),
            _ => (), // By spec we should ignore everything else
        }
    }

    let xt = xt.expect("Missing xt");
    let mut xt: Vec<&str> = xt.split(':').collect();

    dbg!(&xt);

    if xt.len() == 2 {
        bail!("Invalid magnet uri")
    }

    // This should never panic since we just checked the lenght
    let info_hash = match xt.remove(1) {
        "btih" => {
            let btih = xt.remove(1);
            parse_btih(btih)?
        }
        "btmh" => todo!(),
        _ => bail!("Invalid magnet uri"),
    };

    Ok(Magnet {
        info_hash,
        display_name,
        trackers,
        peers,
    })
}

// TODO: errors
fn parse_btih(btih: &str) -> Result<[u8; 20]> {
    let info_hash = if btih.len() == 40 {
        hex::decode(btih)?
    } else if btih.len() == 30 {
        BASE32.decode(btih.as_bytes())?
    } else {
        bail!("Invalid magnet uri");
    };

    Ok(info_hash.try_into().unwrap())
}
