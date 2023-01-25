use std::collections::BTreeSet;

use crate::{Error, Result};
use serde::ser::{self, Serialize, SerializeMap};

pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: std::io::Write,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut writer = Vec::with_capacity(128);
    to_writer(&mut writer, value)?;
    Ok(writer)
}

pub struct Serializer<W> {
    writer: W,
}

impl<W> Serializer<W>
where
    W: std::io::Write + Sized,
{
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

pub struct StructSerializer<'a, W: 'a> {
    serializer: &'a mut Serializer<W>,
    dictionary: BTreeSet<&'static str>,
}

impl<'a, W> StructSerializer<'a, W> {
    pub fn new(serializer: &'a mut Serializer<W>) -> Self {
        Self {
            serializer,
            dictionary: BTreeSet::new(),
        }
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = StructSerializer<'a, W>;
    type SerializeStruct = StructSerializer<'a, W>;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        self.writer.write_all(if value { b"i1e" } else { b"i0e" })?;
        Ok(())
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        write!(self.writer, "i{value}e")?;
        Ok(())
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        write!(self.writer, "i{value}e")?;
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        write!(self.writer, "i{value}e")?;
        Ok(())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        write!(self.writer, "i{value}e")?;
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        write!(self.writer, "i{value}e")?;
        Ok(())
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        write!(self.writer, "i{value}e")?;
        Ok(())
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        write!(self.writer, "i{value}e")?;
        Ok(())
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        write!(self.writer, "i{value}e")?;
        Ok(())
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        let mut buf = [0; 4];
        self.serialize_str(value.encode_utf8(&mut buf))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        write!(self.writer, "{0}:{value}", value.len())?;
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        write!(self.writer, "{0}:", value.len())?;
        self.writer.write_all(value)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.writer.write_all(b"l")?;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.writer.write_all(b"d")?;
        Ok(StructSerializer::new(self))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(None) // There is no reason to pass along the len since we are using a BTreeSet
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
}

impl<'a, W> ser::SerializeSeq for &'a mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_all(b"e")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeTuple for &'a mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_all(b"e")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for &'a mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        self.writer.write_all(b"e")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for &'a mut Serializer<W> {
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a, W> ser::SerializeMap for StructSerializer<'a, W>
where
    W: std::io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        // key.serialize(&mut **self)
        unreachable!()
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        self.serializer.writer.write_all(b"e")?;

        Ok(())
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<(), Self::Error>
    where
        K: Serialize,
        V: Serialize,
    {
        // let s = &mut *self;
        // let serializer = &mut *s.serializer;
        // let key = serializer

        // self.serialize_key(key)?;
        // self.serialize_value(value)

        // let key = self.serializer.serialize(key)?;

        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for StructSerializer<'a, W>
where
    W: std::io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.serialize_entry(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.writer.write_all(b"e")?;
        Ok(())
    }
}

impl<'a, W> ser::SerializeStructVariant for &'a mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_all(b"e")?;
        Ok(())
    }
}
