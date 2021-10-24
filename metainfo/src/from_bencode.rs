use bencode::decode::{AsString, DecodingError, FromBencode, Object};
use url::Url;

use crate::{File, FileKind, Info, MetaInfo};

impl FromBencode for MetaInfo {
    fn bdecode(object: Object) -> Result<Self, DecodingError>
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

        let mut dict_dec = object.dictionary().unwrap();
        while let Some((key, value)) = dict_dec.next_pair()? {
            match key {
                b"announce" => {
                    announce = Some(
                        Url::parse(&String::bdecode(value)?).expect("Invalid announce url"), // TODO: better error handling
                    )
                }
                b"announce-list" => {
                    announce_list = Some(
                        Vec::<Vec<String>>::bdecode(value)?
                            .into_iter()
                            .map(|v| {
                                v.into_iter()
                                    .map(|url| Url::parse(&url).expect("Invalid announce url")) // TODO: better error handling
                                    .collect()
                            })
                            .collect(),
                    )
                }
                b"comment" => comment = Some(String::bdecode(value)?),
                b"created by" => created_by = Some(String::bdecode(value)?),
                b"creation date" => creation_date = Some(u64::bdecode(value)?),
                b"encoding" => encoding = Some(String::bdecode(value)?),
                b"httpseeds" => http_seeds = Some(Vec::bdecode(value)?),
                b"info" => info = Some(Info::bdecode(value)?),
                b"url-list" => {
                    url_list = Some(
                        Vec::<String>::bdecode(value)?
                            .into_iter()
                            .map(|url| Url::parse(&url).expect("Invalid url-list")) // TODO: better error handling
                            .collect(),
                    )
                }
                unknown_field => {
                    return Err(DecodingError::UnexpectedField(
                        String::from_utf8_lossy(unknown_field).to_string(),
                    ));
                }
            }
        }

        // let info = info.ok_or_else(|| DecodingError::Unknown)?;
        let info = info.unwrap();

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
    fn bdecode(object: Object) -> Result<Self, DecodingError>
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

        let mut dict_dec = object.dictionary().unwrap();
        while let Some((key, value)) = dict_dec.next_pair()? {
            match key {
                b"files" => files = Some(Vec::<File>::bdecode(value)?),
                b"length" => length = Some(u64::bdecode(value)?),
                b"md5sum" => md5sum = Some(String::bdecode(value)?),
                b"name" => name = Some(String::bdecode(value)?),
                b"piece length" => piece_length = Some(u64::bdecode(value)?),
                b"pieces" => {
                    pieces = AsString::bdecode(value).map(|bytes| Some(bytes.0))?;
                }
                b"private" => private = Some(u8::bdecode(value)?),
                b"source" => source = Some(String::bdecode(value)?),
                unknown_field => {
                    return Err(DecodingError::UnexpectedField(
                        String::from_utf8_lossy(unknown_field).to_string(),
                    ));
                }
            }
        }

        let name = name.ok_or(DecodingError::Unknown)?;
        let piece_length = piece_length.ok_or(DecodingError::Unknown)?;
        let pieces = pieces.ok_or(DecodingError::Unknown)?;

        Ok(Info {
            name,
            piece_length,
            pieces,
            private,
            source,
            files: if let Some(files) = files {
                FileKind::MultiFile(files)
            } else {
                let length = length.ok_or(DecodingError::Unknown)?;
                FileKind::SingleFile { length, md5sum }
            },
        })
    }
}

impl FromBencode for File {
    fn bdecode(object: Object) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut length = None;
        let mut md5sum = None;
        let mut path = None;

        let mut dict_dec = object.dictionary().unwrap();
        while let Some((key, value)) = dict_dec.next_pair()? {
            match key {
                b"length" => length = Some(u64::bdecode(value)?),
                b"md5sum" => md5sum = Some(String::bdecode(value)?),
                b"path" => path = Some(Vec::<String>::bdecode(value)?),
                unknown_field => {
                    return Err(DecodingError::UnexpectedField(
                        String::from_utf8_lossy(unknown_field).to_string(),
                    ));
                }
            }
        }

        let length = length.ok_or(DecodingError::Unknown)?;
        let path = path.ok_or(DecodingError::Unknown)?;

        Ok(File {
            length,
            md5sum,
            path,
        })
    }
}
