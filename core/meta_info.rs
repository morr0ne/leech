use anyhow::{anyhow, Result};
use bendy::{
    decoding::{Error as DecodingError, FromBencode, Object, ResultExt},
    encoding::{AsString, Error as EncodingError, SingleItemEncoder, ToBencode},
};
use sha1::{digest::FixedOutput, Digest, Sha1};
use std::convert::TryInto;

#[derive(Debug)]
pub struct MetaInfo {
    pub announce: String,
    pub info: Info,
    pub nodes: Option<Vec<Node>>,
    pub encoding: Option<String>,
    pub http_seeds: Option<Vec<String>>,
    pub creation_date: Option<u64>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
}

#[derive(Debug)]
pub struct Node(String, u64);

#[derive(Debug)]
pub enum Info {
    SingleFile {
        name: String,
        length: u64,
        md5sum: Option<String>,
        piece_length: u64,
        pieces: Vec<u8>,
        private: Option<u8>,
    },
    MultiFile {
        name: String,
        files: Vec<File>,
        piece_length: u64,
        pieces: Vec<u8>,
        private: Option<u8>,
    },
}

#[derive(Debug)]
pub struct File {
    pub path: Vec<String>,
    pub length: u64,
    pub md5sum: Option<String>,
}

impl MetaInfo {
    pub fn info_hash(&self) -> Result<[u8; 20]> {
        match self.info.to_bencode() {
            Ok(info) => {
                let mut hasher = Sha1::new();
                hasher.update(info);
                let info_hash: [u8; 20] = hasher.finalize_fixed().try_into()?;

                Ok(info_hash)
            }
            Err(err) => Err(anyhow!(err)),
        }
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

impl FromBencode for MetaInfo {
    const EXPECTED_RECURSION_DEPTH: usize = 2048;

    fn decode_bencode_object(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut announce = None;
        let mut info = None;
        let nodes = None;
        let encoding = None;
        let mut http_seeds = None;
        let mut creation_date = None;
        let mut comment = None;
        let created_by = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some(pair) = dict_dec.next_pair()? {
            match pair {
                (b"announce", value) => {
                    announce = String::decode_bencode_object(value)
                        .context("announce")
                        .map(Some)?;
                }
                (b"info", value) => {
                    info = Info::decode_bencode_object(value)
                        .context("info")
                        .map(Some)?;
                }
                (b"comment", value) => {
                    comment = String::decode_bencode_object(value)
                        .context("comment")
                        .map(Some)?;
                }
                (b"creation date", value) => {
                    creation_date = u64::decode_bencode_object(value)
                        .context("creation_date")
                        .map(Some)?;
                }
                (b"httpseeds", value) => {
                    http_seeds = Vec::decode_bencode_object(value)
                        .context("http_seeds")
                        .map(Some)?;
                }
                (unknown_field, _) => {
                    return Err(DecodingError::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )));
                }
            }
        }

        let announce = announce.ok_or_else(|| DecodingError::missing_field("announce"))?;
        let info = info.ok_or_else(|| DecodingError::missing_field("info"))?;

        Ok(MetaInfo {
            announce,
            info,
            nodes,
            encoding,
            http_seeds,
            creation_date,
            comment,
            created_by,
        })
    }
}

impl FromBencode for Info {
    const EXPECTED_RECURSION_DEPTH: usize = 2048;

    fn decode_bencode_object(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut length = None;
        let mut name = None;
        let mut piece_length = None;
        let mut pieces = None;
        let md5sum = None;
        let private = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some(pair) = dict_dec.next_pair()? {
            match pair {
                (b"length", value) => {
                    length = u64::decode_bencode_object(value)
                        .context("length")
                        .map(Some)?;
                }
                (b"name", value) => {
                    name = String::decode_bencode_object(value)
                        .context("name")
                        .map(Some)?;
                }
                (b"piece length", value) => {
                    piece_length = u64::decode_bencode_object(value)
                        .context("piece_length")
                        .map(Some)?;
                }
                (b"pieces", value) => {
                    pieces = AsString::decode_bencode_object(value)
                        .context("pieces")
                        .map(|bytes| Some(bytes.0))?;
                }
                (unknown_field, _) => {
                    return Err(DecodingError::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )));
                }
            }
        }

        let length = length.ok_or_else(|| DecodingError::missing_field("length"))?;
        let name = name.ok_or_else(|| DecodingError::missing_field("name"))?;
        let piece_length =
            piece_length.ok_or_else(|| DecodingError::missing_field("piece_length"))?;
        let pieces = pieces.ok_or_else(|| DecodingError::missing_field("pieces"))?;

        Ok(Info::SingleFile {
            name,
            length,
            md5sum,
            piece_length,
            pieces,
            private,
        })
    }
}

impl ToBencode for MetaInfo {
    const MAX_DEPTH: usize = Info::MAX_DEPTH + 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), EncodingError> {
        encoder.emit_unsorted_dict(|e| {
            e.emit_pair(b"announce", &self.announce)?;
            e.emit_pair(b"info", &self.info)?;
            if let Some(encoding) = &self.encoding {
                e.emit_pair(b"encoding", encoding)?;
            }
            if let Some(seeds) = &self.http_seeds {
                e.emit_pair(b"httpseeds", seeds)?;
            }
            if let Some(creation_date) = &self.creation_date {
                e.emit_pair(b"creation date", creation_date)?;
            }
            if let Some(comment) = &self.comment {
                e.emit_pair(b"comment", comment)?;
            }
            if let Some(created_by) = &self.created_by {
                e.emit_pair(b"created by", created_by)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl ToBencode for Info {
    const MAX_DEPTH: usize = 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), EncodingError> {
        match self {
            Info::SingleFile {
                name,
                length,
                md5sum,
                piece_length,
                pieces,
                private,
            } => encoder.emit_unsorted_dict(|e| {
                e.emit_pair(b"name", name)?;
                e.emit_pair(b"length", length)?;
                if let Some(sum) = md5sum {
                    e.emit_pair(b"md5sum", sum)?;
                }
                e.emit_pair(b"piece length", piece_length)?;
                e.emit_pair(b"pieces", AsString(pieces))?;
                if let Some(p) = private {
                    e.emit_pair(b"private", p)?;
                }
                Ok(())
            })?,
            Info::MultiFile {
                name,
                files,
                piece_length,
                pieces,
                private,
            } => encoder.emit_unsorted_dict(|e| {
                e.emit_pair(b"name", name)?;
                e.emit_pair(b"files", files)?;
                e.emit_pair(b"piece length", piece_length)?;
                e.emit_pair(b"pieces", AsString(pieces))?;
                if let Some(p) = private {
                    e.emit_pair(b"private", p)?;
                }
                Ok(())
            })?,
        }
        Ok(())
    }
}

impl ToBencode for File {
    const MAX_DEPTH: usize = 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), EncodingError> {
        encoder.emit_unsorted_dict(|e| {
            e.emit_pair(b"path", &self.path)?;
            e.emit_pair(b"length", &self.length)?;
            if let Some(sum) = &self.md5sum {
                e.emit_pair(b"md5sum", sum)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}
