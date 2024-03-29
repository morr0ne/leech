pub static STANDARD_PEERS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "AG" => "Ares",
    "A~" => "Ares",
    "AR" => "Arctic",
    "AV" => "Avicora",
    "AX" => "BitPump",
    "AZ" => "Azureus",
    "BB" => "BitBuddy",
    "BC" => "BitComet",
    "BF" => "Bitflu",
    "BG" => "BTG (uses Rasterbar libtorrent)",
    "BR" => "BitRocket",
    "BS" => "BTSlave",
    "BX" => "~Bittorrent X",
    "CD" => "Enhanced CTorrent",
    "CT" => "CTorrent",
    "DE" => "DelugeTorrent",
    "DP" => "Propagate Data Client",
    "EB" => "EBit",
    "ES" => "electric sheep",
    "FT" => "FoxTorrent",
    "FW" => "FrostWire",
    "FX" => "Freebox BitTorrent",
    "GS" => "GSTorrent",
    "HL" => "Halite",
    "HN" => "Hydranode",
    "KG" => "KGet",
    "KT" => "KTorrent",
    "LH" => "LH=>ABC",
    "LP" => "Lphant",
    "LT" => "libtorrent",
    "lt" => "libTorrent",
    "LW" => "LimeWire",
    "MO" => "MonoTorrent",
    "MP" => "MooPolice",
    "MR" => "Miro",
    "MT" => "MoonlightTorrent",
    "NX" => "Net Transport",
    "PD" => "Pando",
    "qB" => "qBittorrent",
    "QD" => "QQDownload",
    "QT" => "Qt 4 Torrent example",
    "RT" => "Retriever",
    "S~" => "Shareaza alpha/beta",
    "SB" => "~Swiftbit",
    "SS" => "SwarmScope",
    "ST" => "SymTorrent",
    "st" => "sharktorrent",
    "SZ" => "Shareaza",
    "TN" => "TorrentDotNET",
    "TR" => "Transmission",
    "TS" => "Torrentstorm",
    "TT" => "TuoTu",
    "UL" => "uLeecher!",
    "UT" => "µTorrent",
    "UW" => "µTorrent Web",
    "VG" => "Vagaa",
    "WD" => "WebTorrent Desktop",
    "WT" => "BitLet",
    "WW" => "WebTorrent",
    "WY" => "FireTorrent",
    "XL" => "Xunlei",
    "XT" => "XanTorrent",
    "XX" => "Xtorrent",
    "ZT" => "ZipTorrent",
};

pub static SHAD0W_PEERS: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "A" => "ABC",
    "O" => "Osprey Permaseed",
    "Q" => "BTQueue",
    "R" => "Tribler",
    "S" => "Shadow's client",
    "T" => "BitTornado",
    "U" => "UPnP NAT Bit Torrent",
};

/// Helper function to create a valid peer id
pub fn peer_id(client_id: &[u8; 2], version: &[u8; 4]) -> [u8; 20] {
    unsafe {
        array_utils::build_array([b"-", client_id, version, b"-", &rand::random::<[u8; 12]>()])
    }
}
