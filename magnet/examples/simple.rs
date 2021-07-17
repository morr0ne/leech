use anyhow::Result;
use magnet::parse;

fn main() -> Result<()> {
    let url = "magnet:?xt=urn:btih:9d2d5761dba069c786fa492c629e7238b5a4607f&dn=artix-base-openrc-20210426-x86_64.iso&tr=http%3a%2f%2ftracker.artixlinux.org%3a6969%2fannounce&tr=http%3a%2f%2ftorrents.artixlinux.org%3a6969%2fannounce&tr=udp%3a%2f%2ftracker.cyberia.is%3a6969%2fannounce";
    let url = "magnet:?xt=urn:btih:1DD087D88DECD901713AECF9D01C03051415EA78&dn=The.Owl.House.S02E03.Echoes.of.the.Past.1080p.HULU.WEBRip.AAC2.0&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.openbittorrent.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Fopentracker.i2p.rocks%3A6969%2Fannounce&tr=udp%3A%2F%2F47.ip-51-68-199.eu%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.internetwarriors.net%3A1337%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2920%2Fannounce&tr=udp%3A%2F%2Ftracker.pirateparty.gr%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.cyberia.is%3A6969%2Fannounce";
    let url = "magnet:?xt=urn:btih:cab507494d02ebb1178b38f2e9d7be299c86b862&dn=ubuntu-21.04-live-server-amd64.iso&tr=https%3a%2f%2ftorrent.ubuntu.com%2fannounce&tr=https%3a%2f%2fipv6.torrent.ubuntu.com%2fannounce";
    
    
    let magnet = parse(url)?;

    dbg!(magnet);

    Ok(())
}
