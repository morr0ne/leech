use anyhow::{bail, Result};
use bendy::{
    decoding::{Error as DecodingError, FromBencode, Object, ResultExt},
    encoding::{AsString, Error as EncodingError, SingleItemEncoder, ToBencode},
};
use sha1::{digest::FixedOutput, Digest, Sha1};
use std::convert::TryInto;

pub struct MetaInfo {
    pub announce: Option<String>,
    pub announce_list: Option<Vec<String>>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
    pub creation_date: Option<u64>,
    pub encoding: Option<String>,
    pub http_seeds: Option<Vec<String>>,
    pub info: Info,
}

pub enum Info {
    SingleFile {
        length: u64,
        md5sum: Option<String>,
        name: String,
        piece_length: u64,
        pieces: Vec<u8>,
        private: Option<u8>,
    },
    MultiFile {
        files: Vec<File>,
        name: String,
        pieces: Vec<u8>,
        piece_length: u64,
        private: Option<u8>,
    },
}

pub struct File {
    pub length: u64,
    pub md5sum: Option<String>,
    pub path: Vec<String>,
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
            Err(err) => bail!(err),
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
        let mut announce_list = None;
        let mut comment = None;
        let mut created_by = None;
        let mut creation_date = None;
        let mut encoding = None;
        let mut http_seeds = None;
        let mut info = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some((key, value)) = dict_dec.next_pair()? {
            match key {
                b"announce" => announce = Some(String::decode_bencode_object(value)?),
                b"announce-list" => {
                    announce_list = Some(Vec::<String>::decode_bencode_object(value)?)
                }
                b"comment" => comment = Some(String::decode_bencode_object(value)?),
                b"created by" => created_by = Some(String::decode_bencode_object(value)?),
                b"creation date" => creation_date = Some(u64::decode_bencode_object(value)?),
                b"encoding" => encoding = Some(String::decode_bencode_object(value)?),
                b"httpseeds" => http_seeds = Some(Vec::decode_bencode_object(value)?),
                b"info" => info = Some(Info::decode_bencode_object(value)?),
                unknown_field => {
                    return Err(DecodingError::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )));
                }
            }
        }

        let info = info.ok_or_else(|| DecodingError::missing_field("info"))?;

        Ok(MetaInfo {
            announce,
            announce_list,
            comment,
            created_by,
            creation_date,
            encoding,
            http_seeds,
            info,
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
        let mut md5sum = None;
        let mut name = None;
        let mut piece_length = None;
        let mut pieces = None;
        let mut private = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some((key, value)) = dict_dec.next_pair()? {
            match key {
                b"length" => length = Some(u64::decode_bencode_object(value)?),
                b"md5sum" => md5sum = Some(String::decode_bencode_object(value)?),
                b"name" => name = Some(String::decode_bencode_object(value)?),
                b"piece length" => piece_length = Some(u64::decode_bencode_object(value)?),
                b"pieces" => {
                    pieces = AsString::decode_bencode_object(value)
                        .context("pieces")
                        .map(|bytes| Some(bytes.0))?;
                }
                b"private" => private = Some(u8::decode_bencode_object(value)?),
                unknown_field => {
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
        encoder.emit_dict(|mut e| {
            if let Some(announce) = &self.announce {
                e.emit_pair(b"announce", announce)?;
            }
            if let Some(announce_list) = &self.announce_list {
                e.emit_pair(b"announce-list", announce_list)?;
            }
            if let Some(comment) = &self.comment {
                e.emit_pair(b"comment", comment)?;
            }
            if let Some(created_by) = &self.created_by {
                e.emit_pair(b"created by", created_by)?;
            }
            if let Some(creation_date) = &self.creation_date {
                e.emit_pair(b"creation date", creation_date)?;
            }
            if let Some(encoding) = &self.encoding {
                e.emit_pair(b"encoding", encoding)?;
            }
            if let Some(seeds) = &self.http_seeds {
                e.emit_pair(b"httpseeds", seeds)?;
            }
            e.emit_pair(b"info", &self.info)?;
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
            } => encoder.emit_dict(|mut e| {
                e.emit_pair(b"length", length)?;
                if let Some(sum) = md5sum {
                    e.emit_pair(b"md5sum", sum)?;
                }
                e.emit_pair(b"name", name)?;
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
            } => encoder.emit_dict(|mut e| {
                e.emit_pair(b"files", files)?;
                e.emit_pair(b"name", name)?;
                e.emit_pair(b"pieces", AsString(pieces))?;
                e.emit_pair(b"piece length", piece_length)?;
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
        encoder.emit_dict(|mut e| {
            e.emit_pair(b"length", &self.length)?;
            if let Some(sum) = &self.md5sum {
                e.emit_pair(b"md5sum", sum)?;
            }
            e.emit_pair(b"path", &self.path)?;
            Ok(())
        })?;
        Ok(())
    }
}
