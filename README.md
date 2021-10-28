# leech
<!-- [![GitHub license](https://img.shields.io/github/license/morr0ne/leech)](https://github.com/morr0ne/leech/blob/main/LICENSE) -->
[![dependency status](https://deps.rs/repo/github/morr0ne/leech/status.svg)](https://deps.rs/repo/github/morr0ne/leech)

An experimental torrent client/library written in rust.

## Beps status

- [ ] 3 - [The BitTorrent Protocol Specification](https://www.bittorrent.org/beps/bep_0003.html)
    - [x] Bencode encoder/decoder
    - [ ] Metainfo file parsing
    - [ ] Http tracker
    - [ ] Tcp peer protocol
- [ ] 4 - [Known Number Allocations](https://www.bittorrent.org/beps/bep_0004.html)
- [ ] 5 - [DHT Protocol](https://www.bittorrent.org/beps/bep_0005.html)
- [ ] 6 - [Fast Extension](https://www.bittorrent.org/beps/bep_0006.html)
- [x] 7 - [IPv6 Tracker Extension](https://www.bittorrent.org/beps/bep_0007.html)
- [ ] 9 - [Extension for Peers to Send Metadata Files ](https://www.bittorrent.org/beps/bep_0009.html)
    - [ ] Magnet uri parsing
- [ ] 10 - [Extension Protocol](https://www.bittorrent.org/beps/bep_0010.html)
- [ ] 11 - [Peer Exchange (PEX)](https://www.bittorrent.org/beps/bep_0011.html)
- [ ] 12 - [Multitracker Metadata Extension](https://www.bittorrent.org/beps/bep_0012.html)
- [ ] 14 - [Local Service Discovery](https://www.bittorrent.org/beps/bep_0014.html)
- [ ] 15 - [UDP Tracker Protocol](https://www.bittorrent.org/beps/bep_0015.html)
- [ ] 16 - [Superseeding](https://www.bittorrent.org/beps/bep_0016.html)
- [ ] 17 - [HTTP Seeding (Hoffman-style)](https://www.bittorrent.org/beps/bep_0017.html)
- [ ] 19 - [HTTP/FTP Seeding (GetRight-style)](https://www.bittorrent.org/beps/bep_0019.html)
- [ ] 21 - [Extension for Partial Seeds](https://www.bittorrent.org/beps/bep_0021.html)
- [x] 23 - [Tracker Returns Compact Peer Lists](https://www.bittorrent.org/beps/bep_0023.html)
- [ ] 24 - [Tracker Returns External IP](https://www.bittorrent.org/beps/bep_0024.html)
- [ ] 27 - [Private Torrents](https://www.bittorrent.org/beps/bep_0027.html)
- [ ] 29 - [uTorrent transport protocol](https://www.bittorrent.org/beps/bep_0029.html)
- [ ] 30 - [Merkle tree torrent extension](https://www.bittorrent.org/beps/bep_0030.html)
- [ ] 31 - [Tracker Failure Retry Extension](https://www.bittorrent.org/beps/bep_0031.html)
- [ ] 32 - [IPv6 extension for DHT](https://www.bittorrent.org/beps/bep_0032.html)
- [ ] 33 - [DHT scrape](https://www.bittorrent.org/beps/bep_0033.html)
- [ ] 34 - [DNS Tracker Preferences](https://www.bittorrent.org/beps/bep_0034.html)
- [ ] 35 - [Torrent Signing](https://www.bittorrent.org/beps/bep_0035.html)
    - [ ] Signature dictionary parsing
    - [ ] Metainfo file signing
- [ ] 36 - [Torrent RSS feeds](https://www.bittorrent.org/beps/bep_0036.html)
- [ ] 38 - [Finding Local Data Via Torrent File Hints](https://www.bittorrent.org/beps/bep_0038.html)
- [ ] 39 - [Updating Torrents Via Feed URL](https://www.bittorrent.org/beps/bep_0039.html)
- [ ] 40 - [Canonical Peer Prioritys](https://www.bittorrent.org/beps/bep_0040.html)
- [ ] 41 - [UDP Tracker Protocol Extensions](https://www.bittorrent.org/beps/bep_0041.html)
- [ ] 42 - [DHT Security Extension](https://www.bittorrent.org/beps/bep_0042.html)
- [ ] 43 - [Read-only DHT Nodes](https://www.bittorrent.org/beps/bep_0043.html)
- [ ] 44 - [Storing arbitrary data in the DHT](https://www.bittorrent.org/beps/bep_0044.html)
- [ ] 45 - [Multiple-address operation for the BitTorrent DHT](https://www.bittorrent.org/beps/bep_0045.html)
- [ ] 46 - [Updating Torrents Via DHT Mutable Items](https://www.bittorrent.org/beps/bep_0046.html)
- [ ] 47 - [Padding files and extended file attributes](https://www.bittorrent.org/beps/bep_0047.html)
- [ ] 48 - [Tracker Protocol Extension: Scrape](https://www.bittorrent.org/beps/bep_0048.html)
- [ ] 49 - [Distributed Torrent Feeds](https://www.bittorrent.org/beps/bep_0049.html)
- [ ] 50 - [Publish/Subscribe Protocol](https://www.bittorrent.org/beps/bep_0050.html)
- [ ] 51 - [DHT Infohash Indexing](https://www.bittorrent.org/beps/bep_0051.html)
- [ ] 52 - [The BitTorrent Protocol Specification v2](https://www.bittorrent.org/beps/bep_0052.html)
- [ ] 53 - [Magnet URI extension - Select specific file indices for download](https://www.bittorrent.org/beps/bep_0053.html)
- [ ] 54 - [The lt_donthave extension](https://www.bittorrent.org/beps/bep_0054.html)
- [ ] 55 - [Holepunch extension](https://www.bittorrent.org/beps/bep_0055.html)