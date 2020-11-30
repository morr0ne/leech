use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha1::Sha1;

#[derive(Debug, Serialize, Deserialize)]
pub struct MetaInfo {
    pub info: Info,
    pub announce: String,
    pub nodes: Option<Vec<Node>>,
    pub encoding: Option<String>,
    pub httpseeds: Option<Vec<String>>,
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
    #[serde(rename = "creation date")]
    pub creation_date: Option<i64>,
    #[serde(rename = "comment")]
    pub comment: Option<String>,
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node(String, i64);

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Info {
    MultiFile {
        name: String,
        files: Vec<File>,
        #[serde(rename = "piece length")]
        piece_length: i64,
        pieces: ByteBuf,
        private: Option<u8>,
    },
    SingleFile {
        name: String,
        length: i64,
        md5sum: Option<String>,
        #[serde(rename = "piece length")]
        piece_length: i64,
        pieces: ByteBuf,
        private: Option<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    pub path: Vec<String>,
    pub length: i64,
    pub md5sum: Option<String>,
}

impl MetaInfo {
    pub fn info_hash(&self) -> Result<[u8; 20]> {
        let info = serde_bencode::ser::to_bytes(&self.info)?;
        let info_hash: [u8; 20] = Sha1::from(info).digest().bytes();
        Ok(info_hash)
    }
}
