use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::{char, digit1},
    combinator::{eof, map, map_parser},
    multi::length_data,
    sequence::{delimited, terminated},
    IResult,
};

#[derive(Debug)]
pub enum Token<'a> {
    ByteString(&'a [u8]),
    Integer(i64),
    ListStart,
    DictionaryStart,
    End,
    Eof,
}

pub struct Decoder;

impl Decoder {
    pub fn decode_byte_string_raw(bytes: &[u8]) -> IResult<&[u8], &[u8]> {
        length_data(map_parser(
            terminated(digit1, char(':')),
            nom::character::complete::u64,
        ))(bytes)
    }

    pub fn decode_byte_string(bytes: &[u8]) -> IResult<&[u8], Token> {
        map(Self::decode_byte_string_raw, Token::ByteString)(bytes)
    }

    pub fn decode_integer(bytes: &[u8]) -> IResult<&[u8], Token> {
        map(
            map_parser(
                delimited(char('i'), take_until("e"), char('e')),
                nom::character::complete::i64,
            ),
            Token::Integer,
        )(bytes)
    }

    pub fn next_token(bytes: &[u8]) -> IResult<&[u8], Token> {
        alt((
            Self::decode_byte_string,
            Self::decode_integer,
            map(char('l'), |_| Token::ListStart),
            map(char('d'), |_| Token::DictionaryStart),
            map(char('e'), |_| Token::End),
            map(eof, |_| Token::Eof),
        ))(bytes)
    }
}
