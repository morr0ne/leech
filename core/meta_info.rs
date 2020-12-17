use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha1::{digest::FixedOutput, Digest, Sha1};
use std::convert::TryInto;

#[derive(Serialize, Deserialize)]
pub struct MetaInfo {
    pub info: Info,
    pub announce: String,
    pub nodes: Option<Vec<Node>>,
    pub encoding: Option<String>,
    pub httpseeds: Option<Vec<String>>,
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
    #[serde(rename = "creation date")]
    pub creation_date: Option<u64>,
    #[serde(rename = "comment")]
    pub comment: Option<String>,
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Node(String, u64);

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Info {
    MultiFile {
        name: String,
        files: Vec<File>,
        #[serde(rename = "piece length")]
        piece_length: u64,
        pieces: ByteBuf,
        private: Option<u8>,
    },
    SingleFile {
        name: String,
        length: u64,
        md5sum: Option<String>,
        #[serde(rename = "piece length")]
        piece_length: u64,
        pieces: ByteBuf,
        private: Option<u8>,
    },
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub path: Vec<String>,
    pub length: u64,
    pub md5sum: Option<String>,
}

impl MetaInfo {
    pub fn info_hash(&self) -> Result<[u8; 20]> {
        let info = serde_bencode::ser::to_bytes(&self.info)?;

        let mut hasher = Sha1::new();
        hasher.update(info);
        let info_hash: [u8; 20] = hasher.finalize_fixed().try_into()?;

        Ok(info_hash)
    }

    pub fn length(&self) -> u64 {
        match self.info {
            Info::SingleFile { length, .. } => length,
            Info::MultiFile { ref files, .. } => {
                files.iter().fold(0, |index, file| index + file.length)
            }
        }
    }
}
