use bento::{AsString, DecodingError, Encoder, FromBencode, Object, ToBencode};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use sha1::{digest::FixedOutput, Digest, Sha1};
use std::convert::TryInto;
use url::Url;

// pub use bento;

/// Dictionary containg information about the torrent
#[derive(Debug, Deserialize, Serialize)]
pub struct MetaInfo {
    /// The announce url of the tracker.
    /// According to the specification this is always set.
    /// In the real world most torrents ditch it in favor of announce list or trackless peers
    ///
    /// The url supports http tracking via get requests and udp tracking. It is worth noting that many trackers will accept either protocols regardless of the one specified
    pub announce: Option<Url>,
    /// A list of list of announce urls.
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<Url>>>,
    /// An optional comment about this torrent
    pub comment: Option<String>,
    /// Name of version of the program used to create the torrent
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
    /// Time of creation represented in millisecond since [UNIX epoch][`std::time::UNIX_EPOCH`]
    #[serde(rename = "creation date")]
    pub creation_date: Option<u64>,
    /// The encoding format used by [pieces][`Info::pieces`]
    pub encoding: Option<String>,
    /// Unknown field
    pub http_seeds: Option<Vec<String>>,
    /// A dictionary containing information about the file(s) of the torrent
    pub info: Info,
    // TODO: docs
    pub url_list: Option<Vec<Url>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    /// The name of the file or directory to store multiple files, respecting this field is not mandatory
    pub name: String,
    /// The number of bytes in each piece
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    /// String consisting of the concatenation of all 20-byte SHA1 hash values, one per piece
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
    /// When set to 1 clients should only announce their presence via the tracker specified by the torrent
    pub private: Option<u8>,
    /// Unknown field
    pub source: Option<String>,
    #[serde(flatten)]
    pub files: FileKind,
}

/// A dictionary containing information about the file(s) of the torrent
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, untagged)]
pub enum FileKind {
    // Information about multiple files
    MultiFile {
        /// A list of dictionaries, each containing information about one file
        files: Vec<File>,
    },

    /// Information about a single file
    SingleFile {
        /// Length of the file in bytes
        length: u64,
        /// MD5 sum of the file
        md5sum: Option<String>,
    },
}

/// Dictionary containing information about a file
#[derive(Debug, Deserialize, Serialize)]
pub struct File {
    /// Length of the file in bytes
    pub length: u64,
    /// MD5 sum of the file
    pub md5sum: Option<String>,
    /// A list where each element corresponds to either a directory name or (in the case of the final element) the filename
    pub path: Vec<String>,
}

impl MetaInfo {
    /// Returns the SHA-1 hash of the info dictionary
    pub fn info_hash(&self) -> Result<[u8; 20]> {
        // Following spec, we first convert back into bencode
        // let info = self.info.to_bencode()?; // TODO: better error handeling
        let info = bencode::to_vec(&self.info)?;

        let mut hasher = Sha1::new();
        hasher.update(&info);
        let info_hash: [u8; 20] = hasher.finalize_fixed().try_into().unwrap(); // NOTE: This shouldn't fail in theory but unless const generics is stabilized it has to stay that way

        Ok(info_hash)
    }

    pub fn length(&self) -> u64 {
        match &self.info.files {
            FileKind::SingleFile { length, .. } => *length, // TODO: probably a better way to do this
            FileKind::MultiFile { files } => {
                files.iter().fold(0, |index, file| index + file.length)
            }
        }
    }
}

// impl FromBencode for MetaInfo {
//     fn decode(object: Object<'_, '_>) -> Result<Self, DecodingError>
//     where
//         Self: Sized,
//     {
//         let mut announce = None;
//         let mut announce_list = None;
//         let mut comment = None;
//         let mut created_by = None;
//         let mut creation_date = None;
//         let mut encoding = None;
//         let mut http_seeds = None;
//         let mut info = None;
//         let mut url_list = None;

//         let mut dict_dec = object.try_dictionary()?;
//         while let Some((key, value)) = dict_dec.next_pair()? {
//             match key {
//                 b"announce" => announce = value.decode()?,
//                 b"announce-list" => announce_list = value.decode()?,
//                 b"comment" => comment = value.decode()?,
//                 b"created by" => created_by = value.decode()?,
//                 b"creation date" => creation_date = value.decode()?,
//                 b"encoding" => encoding = value.decode()?,
//                 b"httpseeds" => http_seeds = value.decode()?,
//                 b"info" => info = value.decode()?,
//                 b"url-list" => url_list = value.decode()?,
//                 _unknown_field => value.skip()?,
//             }
//         }

//         Ok(MetaInfo {
//             announce,
//             announce_list,
//             comment,
//             created_by,
//             creation_date,
//             encoding,
//             http_seeds,
//             info: info.ok_or(DecodingError::MissingField { field: "info" })?,
//             url_list,
//         })
//     }
// }

// impl FromBencode for Info {
//     fn decode(object: Object<'_, '_>) -> Result<Self, DecodingError>
//     where
//         Self: Sized,
//     {
//         let mut files = None;
//         let mut length = None;
//         let mut md5sum = None;
//         let mut name = None;
//         let mut piece_length = None;
//         let mut pieces = None;
//         let mut private = None;
//         let mut source = None;

//         let mut dict_dec = object.try_dictionary()?;
//         while let Some((key, value)) = dict_dec.next_pair()? {
//             match key {
//                 b"files" => files = value.decode()?,
//                 b"length" => length = value.decode()?,
//                 b"md5sum" => md5sum = value.decode()?,
//                 b"name" => name = value.decode()?,
//                 b"piece length" => piece_length = value.decode()?,
//                 b"pieces" => {
//                     let p = AsString::decode(value)?;
//                     if p.len() % 20 == 0 {
//                         pieces = Some(p);
//                     } else {
//                         return Err(DecodingError::Unknown);
//                     }
//                 }
//                 b"private" => private = value.decode()?,
//                 b"source" => source = value.decode()?,
//                 unknown_field => {
//                     return Err(DecodingError::UnexpectedField {
//                         field: String::from_utf8_lossy(unknown_field).to_string(),
//                     });
//                 }
//             }
//         }

//         Ok(Info {
//             name: name.ok_or(DecodingError::MissingField { field: "name" })?,
//             piece_length: piece_length.ok_or(DecodingError::MissingField {
//                 field: "piece_length",
//             })?,
//             pieces: pieces.ok_or(DecodingError::MissingField { field: "pieces" })?,
//             private,
//             source,
//             files: if let Some(files) = files {
//                 FileKind::MultiFile(files)
//             } else {
//                 let length = length.ok_or(DecodingError::MissingField { field: "length" })?;
//                 FileKind::SingleFile { length, md5sum }
//             },
//         })
//     }
// }

// impl FromBencode for File {
//     fn decode(object: Object<'_, '_>) -> Result<Self, DecodingError>
//     where
//         Self: Sized,
//     {
//         let mut length = None;
//         let mut md5sum = None;
//         let mut path = None;

//         let mut dict_dec = object.try_dictionary()?;
//         while let Some((key, value)) = dict_dec.next_pair()? {
//             match key {
//                 b"length" => length = value.decode()?,
//                 b"md5sum" => md5sum = value.decode()?,
//                 b"path" => path = value.decode()?,
//                 unknown_field => {
//                     return Err(DecodingError::UnexpectedField {
//                         field: String::from_utf8_lossy(unknown_field).to_string(),
//                     });
//                 }
//             }
//         }

//         Ok(File {
//             length: length.ok_or(DecodingError::MissingField { field: "length" })?,
//             md5sum,
//             path: path.ok_or(DecodingError::MissingField { field: "path" })?,
//         })
//     }
// }

impl ToBencode for MetaInfo {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.emit_dictionary(|mut e| {
            if let Some(announce) = &self.announce {
                e.emit_pair(b"announce", announce.to_string());
            }
            if let Some(announce_list) = &self.announce_list {
                e.emit_pair(b"announce-list", announce_list);
            }
            if let Some(comment) = &self.comment {
                e.emit_pair(b"comment", comment);
            }
            if let Some(created_by) = &self.created_by {
                e.emit_pair(b"created by", created_by);
            }
            if let Some(creation_date) = &self.creation_date {
                e.emit_pair(b"creation date", creation_date);
            }
            if let Some(encoding) = &self.encoding {
                e.emit_pair(b"encoding", encoding);
            }
            if let Some(seeds) = &self.http_seeds {
                e.emit_pair(b"httpseeds", seeds);
            }
            if let Some(url_list) = &self.url_list {
                e.emit_pair(b"url-list", url_list);
            }
            e.emit_pair(b"info", &self.info);
        });
    }
}

impl ToBencode for Info {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.emit_dictionary(|mut e| {
            match &self.files {
                FileKind::MultiFile { files } => e.emit_pair(b"files", files),
                FileKind::SingleFile { length, md5sum } => {
                    e.emit_pair(b"length", length);
                    if let Some(sum) = md5sum {
                        e.emit_pair(b"md5sum", sum);
                    }
                }
            }

            e.emit_pair(b"name", &self.name);
            e.emit_pair(b"piece length", self.piece_length);
            e.emit_pair(b"pieces", AsString(self.pieces.clone())); // FIXME: Don't clone
            if let Some(private) = self.private {
                e.emit_pair(b"private", private);
            }
            if let Some(source) = &self.source {
                e.emit_pair(b"source", source);
            }
        });
    }
}

impl ToBencode for File {
    fn encode(&self, encoder: &mut Encoder) {
        encoder.emit_dictionary(|mut e| {
            e.emit_pair(b"length", &self.length);
            if let Some(sum) = &self.md5sum {
                e.emit_pair(b"md5sum", sum);
            }
            e.emit_pair(b"path", &self.path);
        });
    }
}
