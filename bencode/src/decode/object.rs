use super::decoder::{DictionaryDecoder, ListDecoder};

pub enum Object<'obj, 'de: 'obj> {
    ByteString(&'de [u8]),
    Integer(&'de [u8]),
    List(ListDecoder<'obj, 'de>),
    Dictionary(DictionaryDecoder<'obj, 'de>),
}

impl<'obj, 'de: 'obj> Object<'obj, 'de> {
    pub const fn byte_string(self) -> Option<&'de [u8]> {
        match self {
            Object::ByteString(byte_string) => Some(byte_string),
            _ => None,
        }
    }

    pub const fn integer(self) -> Option<&'de [u8]> {
        match self {
            Object::Integer(integer) => Some(integer),
            _ => None,
        }
    }

    pub const fn list(self) -> Option<ListDecoder<'obj, 'de>> {
        match self {
            Object::List(list_decoder) => Some(list_decoder),
            _ => None,
        }
    }

    pub const fn dictionary(self) -> Option<DictionaryDecoder<'obj, 'de>> {
        match self {
            Object::Dictionary(dictionary_decoder) => Some(dictionary_decoder),
            _ => None,
        }
    }
}
