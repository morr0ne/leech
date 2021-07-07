use anyhow::Result;
use bendy::{
    decoding::{Error as DecodingError, FromBencode, Object, ResultExt},
    encoding::{AsString, Error as EncodingError, SingleItemEncoder, ToBencode},
};
use sha1::{digest::FixedOutput, Digest, Sha1};
use std::convert::TryInto;
use url::Url;

pub use bendy;

/// Dictionary containg information about the torrent
#[derive(Debug)]
pub struct MetaInfo {
    /// The announce url of the tracker.
    /// According to the specification this is always set.
    /// In the real world most torrents ditch it in favor of announce list or trackless peers
    ///
    /// The url supports http tracking via get requests and udp tracking. It is worth noting that many trackers will accept either protocols regardless of the one specified
    pub announce: Option<Url>,
    /// A list of list of announce urls.
    pub announce_list: Option<Vec<Vec<Url>>>,
    /// An optional comment about this torrent
    pub comment: Option<String>,
    /// Name of version of the program used to create the torrent
    pub created_by: Option<String>,
    /// Time of creation represented in millisecond since [UNIX epoch][`std::time::UNIX_EPOCH`]
    pub creation_date: Option<u64>,
    /// The encoding format used by [pieces][`Info::SingleFile::pieces`]
    pub encoding: Option<String>,
    /// Unknown field
    pub http_seeds: Option<Vec<String>>,
    /// A dictionary containing information about the file(s) of the torrent
    pub info: Info,
    // TODO: docs
    pub url_list: Option<Vec<Url>>,
}

#[derive(Debug)]
pub struct Info {
    /// The name of the file or directory to store multiple files, respecting this field is not mandatory
    name: String,
    /// The number of bytes in each piece
    piece_length: u64,
    /// String consisting of the concatenation of all 20-byte SHA1 hash values, one per piece
    pieces: Vec<u8>,
    /// When set to 1 clients should only announce their presence via the tracker specified by the torrent
    private: Option<u8>,
    /// Unknown field
    source: Option<String>,
    files: FileKind,
}

/// A dictionary containing information about the file(s) of the torrent
#[derive(Debug)]
pub enum FileKind {
    /// Information about a single file
    SingleFile {
        /// Length of the file in bytes
        length: u64,
        /// MD5 sum of the file
        md5sum: Option<String>,
    },
    /// A list of dictionaries, each containing information about one file
    MultiFile(Vec<File>),
}

/// Dictionary containing information about a file
#[derive(Debug)]
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
        let info = self.info.to_bencode()?; // TODO: better error handeling

        let mut hasher = Sha1::new();
        hasher.update(info);
        let info_hash: [u8; 20] = hasher.finalize_fixed().try_into()?;

        Ok(info_hash)
    }

    pub fn length(&self) -> u64 {
        match &self.info.files {
            FileKind::SingleFile { length, .. } => length.clone(), // TODO: probably a better way to do this
            FileKind::MultiFile(files) => files.iter().fold(0, |index, file| index + file.length),
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
        let mut url_list = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some((key, value)) = dict_dec.next_pair()? {
            match key {
                b"announce" => {
                    announce = Some(
                        Url::parse(&String::decode_bencode_object(value).context("announce")?)
                            .expect("Invalid announce url"), // TODO: better error handling
                    )
                }
                b"announce-list" => {
                    announce_list = Some(
                        Vec::<Vec<String>>::decode_bencode_object(value)
                            .context("announce-list")?
                            .into_iter()
                            .map(|v| {
                                v.into_iter()
                                    .map(|url| Url::parse(&url).expect("Invalid announce url")) // TODO: better error handling
                                    .collect()
                            })
                            .collect(),
                    )
                }
                b"comment" => {
                    comment = Some(String::decode_bencode_object(value).context("comment")?)
                }
                b"created by" => {
                    created_by = Some(String::decode_bencode_object(value).context("created by")?)
                }
                b"creation date" => {
                    creation_date =
                        Some(u64::decode_bencode_object(value).context("creation date")?)
                }
                b"encoding" => {
                    encoding = Some(String::decode_bencode_object(value).context("encoding")?)
                }
                b"httpseeds" => {
                    http_seeds = Some(Vec::decode_bencode_object(value).context("httpseeds")?)
                }
                b"info" => info = Some(Info::decode_bencode_object(value).context("info")?),
                b"url-list" => {
                    url_list = Some(
                        Vec::<String>::decode_bencode_object(value)
                            .context("url-list")?
                            .into_iter()
                            .map(|url| Url::parse(&url).expect("Invalid url-list")) // TODO: better error handling
                            .collect(),
                    )
                }
                unknown_field => {
                    return Err(DecodingError::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )))
                    .context("Metainfo");
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
            url_list,
        })
    }
}

impl FromBencode for Info {
    const EXPECTED_RECURSION_DEPTH: usize = 2048;

    fn decode_bencode_object(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut files = None;
        let mut length = None;
        let mut md5sum = None;
        let mut name = None;
        let mut piece_length = None;
        let mut pieces = None;
        let mut private = None;
        let mut source = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some((key, value)) = dict_dec.next_pair()? {
            match key {
                b"files" => files = Some(Vec::<File>::decode_bencode_object(value)?),
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
                b"source" => source = Some(String::decode_bencode_object(value)?),
                unknown_field => {
                    return Err(DecodingError::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )))
                    .context("Info");
                }
            }
        }

        let name = name.ok_or_else(|| DecodingError::missing_field("name"))?;
        let piece_length =
            piece_length.ok_or_else(|| DecodingError::missing_field("piece_length"))?;
        let pieces = pieces.ok_or_else(|| DecodingError::missing_field("pieces"))?;

        Ok(Info {
            name,
            piece_length,
            pieces,
            private,
            source,
            files: if let Some(files) = files {
                FileKind::MultiFile(files)
            } else {
                let length = length.ok_or_else(|| DecodingError::missing_field("length"))?;
                FileKind::SingleFile { length, md5sum }
            },
        })
    }
}

impl FromBencode for File {
    const EXPECTED_RECURSION_DEPTH: usize = 0;

    fn decode_bencode_object(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut length = None;
        let mut md5sum = None;
        let mut path = None;

        let mut dict_dec = object.try_into_dictionary()?;
        while let Some((key, value)) = dict_dec.next_pair()? {
            match key {
                b"length" => length = Some(u64::decode_bencode_object(value).context("length")?),
                b"md5sum" => md5sum = Some(String::decode_bencode_object(value).context("md5sum")?),
                b"path" => {
                    path = Some(Vec::<String>::decode_bencode_object(value).context("path")?)
                }
                unknown_field => {
                    return Err(DecodingError::unexpected_field(String::from_utf8_lossy(
                        unknown_field,
                    )));
                }
            }
        }

        let length = length.ok_or_else(|| DecodingError::missing_field("length"))?;
        let path = path.ok_or_else(|| DecodingError::missing_field("path"))?;

        Ok(File {
            length,
            md5sum,
            path,
        })
    }
}

impl ToBencode for MetaInfo {
    const MAX_DEPTH: usize = Info::MAX_DEPTH + 1;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), EncodingError> {
        encoder.emit_dict(|mut e| {
            if let Some(announce) = &self.announce {
                e.emit_pair(b"announce", announce.to_string())?;
            }
            if let Some(announce_list) = &self.announce_list {
                e.emit_pair::<Vec<String>>(
                    b"announce-list",
                    announce_list
                        .into_iter()
                        .map(|v| v.into_iter().map(|url| url.to_string()).collect())
                        .collect(),
                )?;
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
            if let Some(url_list) = &self.url_list {
                e.emit_pair::<Vec<String>>(
                    b"url-list",
                    url_list.into_iter().map(|url| url.to_string()).collect(),
                )?;
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
        encoder.emit_dict(|mut e| {
            match &self.files {
                FileKind::MultiFile(files) => e.emit_pair(b"files", files)?,
                FileKind::SingleFile { length, md5sum } => {
                    e.emit_pair(b"length", length)?;
                    if let Some(sum) = md5sum {
                        e.emit_pair(b"md5sum", sum)?;
                    }
                }
            }

            e.emit_pair(b"name", &self.name)?;
            e.emit_pair(b"piece length", self.piece_length)?;
            e.emit_pair(b"pieces", AsString(&self.pieces))?;
            if let Some(private) = self.private {
                e.emit_pair(b"private", private)?;
            }
            if let Some(source) = &self.source {
                e.emit_pair(b"source", source)?;
            }

            Ok(())
        })?;

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
