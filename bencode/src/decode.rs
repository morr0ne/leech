use indexmap::IndexMap;
use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::{char, digit1},
    combinator::{map, map_parser},
    multi::{fold_many0, length_data, many0},
    sequence::{delimited, pair, terminated},
    Finish, IResult,
};

use crate::Value;

pub fn decode(data: &[u8]) -> Result<Value, nom::error::Error<&[u8]>> {
    Ok(decode_any(data).finish()?.1)
}

pub fn decode_any(data: &[u8]) -> IResult<&[u8], Value> {
    alt((
        decode_byte_string,
        decode_integer,
        decode_lists,
        decode_dictionaries,
    ))(data)
}

pub fn decode_byte_string_raw(data: &[u8]) -> IResult<&[u8], &[u8]> {
    length_data(map_parser(
        terminated(digit1, char(':')),
        nom::character::complete::u64,
    ))(data)
}

pub fn decode_byte_string(data: &[u8]) -> IResult<&[u8], Value> {
    map(decode_byte_string_raw, Value::ByteString)(data)
}

pub fn decode_integer_raw(data: &[u8]) -> IResult<&[u8], i64> {
    map_parser(
        delimited(char('i'), take_until("e"), char('e')),
        nom::character::complete::i64,
    )(data)
}

pub fn decode_integer(data: &[u8]) -> IResult<&[u8], Value> {
    map(decode_integer_raw, Value::Integer)(data)
}

pub fn decode_lists(data: &[u8]) -> IResult<&[u8], Value> {
    map(
        delimited(char('l'), many0(decode_any), char('e')),
        Value::List,
    )(data)
}

pub fn decode_dictionaries(data: &[u8]) -> IResult<&[u8], Value> {
    map(
        delimited(
            char('d'),
            fold_many0(
                pair(decode_byte_string_raw, decode_any),
                IndexMap::new,
                |mut dict, (key, value)| {
                    dict.insert(key, value);
                    dict
                },
            ),
            char('e'),
        ),
        Value::Dictionary,
    )(data)
}
