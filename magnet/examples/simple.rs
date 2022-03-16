use color_eyre::eyre::Result;
use magnet::parse;

fn main() -> Result<()> {
    color_eyre::install()?;
    let _url = "magnet:?xt=urn:btih:9d2d5761dba069c786fa492c629e7238b5a4607f&dn=artix-base-openrc-20210426-x86_64.iso&tr=http%3a%2f%2ftracker.artixlinux.org%3a6969%2fannounce&tr=http%3a%2f%2ftorrents.artixlinux.org%3a6969%2fannounce&tr=udp%3a%2f%2ftracker.cyberia.is%3a6969%2fannounce";
    let url = "magnet:?xt=urn:btih:cab507494d02ebb1178b38f2e9d7be299c86b862&dn=ubuntu-21.04-live-server-amd64.iso&tr=https%3a%2f%2ftorrent.ubuntu.com%2fannounce&tr=https%3a%2f%2fipv6.torrent.ubuntu.com%2fannounce";

    let magnet = parse(url)?;

    dbg!(magnet);

    Ok(())
}
