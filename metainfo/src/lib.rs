use anyhow::{bail, Result};
use bendy::{
    decoding::{Error as DecodingError, FromBencode, Object, ResultExt},
    encoding::{AsString, Error as EncodingError, SingleItemEncoder, ToBencode},
};
use sha1::{digest::FixedOutput, Digest, Sha1};
use std::convert::TryInto;
use url::Url;

pub use bendy;

/// Dictionary containg vital information about the torrent
#[derive(Debug)]
pub struct MetaInfo {
    /// The announce url of the tracker.
    /// According to the specification this is always set.
    /// In the real world most torrents ditch it in favor of announce list or trackless peers
    ///
    /// The url supports http tracking via get requests and udp tracking. It is worth noting that many trackers will accept either protocols regardless of the one specified
    pub announce: Option<UrlWrapper>,
    /// A list of list of announce urls.
    pub announce_list: Option<Vec<Vec<UrlWrapper>>>,
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
}

/// A dictionary containing information about the file(s) of the torrent
#[derive(Debug)]
pub enum Info {
    /// Information about a single file
    SingleFile {
        /// Length of the file in bytes
        length: u64,
        /// MD5 sum of the file
        md5sum: Option<String>,
        /// The name of the file, respecting this field is not mandatory
        name: String,
        /// The number of bytes in each piece
        piece_length: u64,
        /// String consisting of the concatenation of all 20-byte SHA1 hash values, one per piece
        pieces: Vec<u8>,
        /// When set to 1 clients should only announce their presence via the tracker specified by the torrent
        private: Option<u8>,
        /// Unknown field
        source: Option<String>,
    },
    /// Information about multiple files
    MultiFile {
        /// A list of dictionaries, each containing information about one file
        files: Vec<File>,
        /// The name of the directory in which to store the files, respecting this field is not mandatory
        name: String,
        /// The number of bytes in each piece
        piece_length: u64,
        /// String consisting of the concatenation of all 20-byte SHA1 hash values, one per piece
        pieces: Vec<u8>,
        /// When set to 1 clients should only announce their presence via the tracker specified by the torrent
        private: Option<u8>,
        /// Unknown field
        source: Option<String>,
    },
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

#[derive(Debug)]
pub struct UrlWrapper(pub Url);

impl MetaInfo {
    /// Returns the SHA-1 hash of the info dictionary
    pub fn info_hash(&self) -> Result<[u8; 20]> {
        // Following spec, we first convert back into bencode
        match self.info.to_bencode() {
            Ok(info) => {
                // and then calculate the sha1
                let mut hasher = Sha1::new();
                hasher.update(info);
                let info_hash: [u8; 20] = hasher.finalize_fixed().try_into()?;

                Ok(info_hash)
            }
            Err(err) => bail!(err), // TODO: better error handeling
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
                b"announce" => {
                    announce = Some(UrlWrapper::decode_bencode_object(value).context("announce")?)
                }
                b"announce-list" => {
                    announce_list = Some(
                        Vec::<Vec<UrlWrapper>>::decode_bencode_object(value)
                            .context("announce-list")?,
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
                    )));
                }
            }
        }

        let name = name.ok_or_else(|| DecodingError::missing_field("name"))?;
        let piece_length =
            piece_length.ok_or_else(|| DecodingError::missing_field("piece_length"))?;
        let pieces = pieces.ok_or_else(|| DecodingError::missing_field("pieces"))?;

        if let Some(files) = files {
            Ok(Info::MultiFile {
                files,
                name,
                piece_length,
                pieces,
                private,
                source,
            })
        } else {
            let length = length.ok_or_else(|| DecodingError::missing_field("length"))?;
            Ok(Info::SingleFile {
                name,
                length,
                md5sum,
                piece_length,
                pieces,
                private,
                source,
            })
        }
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
                source,
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
                if let Some(p) = source {
                    e.emit_pair(b"source", p)?;
                }
                Ok(())
            })?,
            Info::MultiFile {
                name,
                files,
                piece_length,
                pieces,
                private,
                source,
            } => encoder.emit_dict(|mut e| {
                e.emit_pair(b"files", files)?;
                e.emit_pair(b"name", name)?;
                e.emit_pair(b"piece length", piece_length)?;
                e.emit_pair(b"pieces", AsString(pieces))?;
                if let Some(p) = private {
                    e.emit_pair(b"private", p)?;
                }
                if let Some(p) = source {
                    e.emit_pair(b"source", p)?;
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

impl From<UrlWrapper> for Url {
    fn from(wrapper: UrlWrapper) -> Self {
        wrapper.0
    }
}

impl From<Url> for UrlWrapper {
    fn from(url: Url) -> Self {
        UrlWrapper(url)
    }
}

impl FromBencode for UrlWrapper {
    const EXPECTED_RECURSION_DEPTH: usize = 2048;

    fn decode_bencode_object(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        Ok(UrlWrapper(
            Url::parse(&String::decode_bencode_object(object)?).unwrap(), // TODO: better error handeling
        ))
    }
}

impl ToBencode for UrlWrapper {
    const MAX_DEPTH: usize = 0;

    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), EncodingError> {
        encoder.emit_str(self.0.as_str())
    }
}
