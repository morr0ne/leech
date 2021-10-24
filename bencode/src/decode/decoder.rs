use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::{char, digit1},
    combinator::{map, map_parser, opt},
    multi::length_data,
    sequence::{delimited, terminated},
    Finish, IResult,
};

use super::{error::DecodingError, object::Object};

enum Token<'a> {
    ByteString(&'a [u8]),
    Integer(&'a [u8]),
    ListStart,
    DictionaryStart,
    End,
}

pub struct Decoder<'a> {
    bytes: &'a [u8],
}

pub struct ListDecoder<'obj, 'de: 'obj>(&'obj mut Decoder<'de>);

pub struct DictionaryDecoder<'obj, 'de: 'obj>(&'obj mut Decoder<'de>);

impl<'de> Decoder<'de> {
    pub const fn new(bytes: &'de [u8]) -> Self {
        Self { bytes }
    }

    fn decode_byte_string(bytes: &[u8]) -> IResult<&[u8], Token> {
        map(
            length_data(map_parser(
                terminated(digit1, char(':')),
                nom::character::complete::u64,
            )),
            Token::ByteString,
        )(bytes)
    }

    fn decode_integer(bytes: &[u8]) -> IResult<&[u8], Token> {
        map(
            map_parser(delimited(char('i'), take_until("e"), char('e')), digit1),
            Token::Integer,
        )(bytes)
    }

    fn next_token(&mut self) -> Result<Option<Token<'de>>, DecodingError> {
        let (bytes, token) = opt(alt((
            Self::decode_byte_string,
            Self::decode_integer,
            map(char('l'), |_| Token::ListStart),
            map(char('d'), |_| Token::DictionaryStart),
            map(char('e'), |_| Token::End),
        )))(self.bytes)
        .finish()
        .map_err(|_| DecodingError::Unknown)?; // TODO: Map to an actual error

        self.bytes = bytes;
        Ok(token)
    }

    pub fn next_object<'obj>(&'obj mut self) -> Result<Option<Object<'obj, 'de>>, DecodingError> {
        Ok(match self.next_token()? {
            None | Some(Token::End) => None,
            Some(Token::ByteString(byte_string)) => Some(Object::ByteString(byte_string)),
            Some(Token::Integer(integer)) => Some(Object::Integer(integer)),
            Some(Token::ListStart) => Some(Object::List(ListDecoder::new(self))),
            Some(Token::DictionaryStart) => Some(Object::Dictionary(DictionaryDecoder::new(self))),
        })
    }
}

impl<'obj, 'de: 'obj> ListDecoder<'obj, 'de> {
    pub fn new(decoder: &'obj mut Decoder<'de>) -> Self {
        Self(decoder)
    }

    pub fn next_object<'item>(
        &'item mut self,
    ) -> Result<Option<Object<'item, 'de>>, DecodingError> {
        let item = self.0.next_object()?;
        if item.is_none() {
            return Ok(None);
        }

        Ok(item)
    }
}

impl<'obj, 'de: 'obj> DictionaryDecoder<'obj, 'de> {
    pub fn new(decoder: &'obj mut Decoder<'de>) -> Self {
        Self(decoder)
    }

    pub fn next_pair<'item>(
        &'item mut self,
    ) -> Result<Option<(&'de [u8], Object<'item, 'de>)>, DecodingError> {
        Ok(if let Some(Object::ByteString(k)) = self.0.next_object()? {
            let v = self.0.next_object()?.unwrap();
            Some((k, v))
        } else {
            None
        })
    }
}
