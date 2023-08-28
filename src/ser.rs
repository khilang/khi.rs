//! Serde serializer for the Khi data format.

use std::str::FromStr;
use hex::ToHex;
use numtoa::NumToA;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeTuple};
use serde::{Deserialize, Serialize};
use crate::parse::ParsedExpression;
use crate::{Component, Dictionary, Directive, Expression, Table, Text};
use crate::model::{SimpleStructure as MStructure};
use crate::fmt::FastFormatter;

/// Serialize a data type to a Khi structure.
pub fn to_expression<T: Serialize>(t: T) -> Result<impl Expression<_, _, _, _, _>> {
    let serializer = StructureSerializer::new();
    t.serialize(serializer)?;
    serializer.done()
}

/// Serialize a data type to a Khi structure string.
pub fn to_string<T: Serialize>(t: T) -> Result<String> {
    let structure = to_expression(t);
    let out = structure.format_compact();
    Ok(out)
}

trait SerializeStructure: Serialize {

    fn serialize_as_auk_structure(&self) -> impl Expression<_, _, _, _, _>;

    fn serialize_to_auk_string(&self) -> String;

}

impl <S: Serialize> SerializeStructure for S {

    fn serialize_as_auk_structure(&self) -> impl Expression<_, _, _, _, _> {
        input.serialize(StructureSerializer::new())
    }

    fn serialize_to_auk_string(&self) -> String {
        self.serialize_as_auk_structure().encode()
    }

}

pub type Result<T> = std::result::Result<T, Error>;

/// Khi serialization error.
pub enum Error {
    /// Unable to convert key structure to text.
    NonTextKey,
}

pub struct StructureSerializer {
    result: MStructure,
    buffer: [u8; 20],
}

impl StructureSerializer {

    pub fn new() -> Self {
        Self { result: MStructure::Empty, buffer: [0u8; 20] }
    }

    pub fn serialize_num<T: NumToA<T>>(&mut self, v: T) -> () {
        let str = v.numtoa_str(10, &mut self.buffer);
        self.result.push_str(str);

        ()
    }

    pub fn done() -> impl Expression<_, _, _, _, _> {

    }

}

impl serde::Serializer for &mut StructureSerializer {

    type Ok = ();

    type Error = Error;

    type SerializeSeq = ();

    type SerializeTuple = ();

    type SerializeTupleStruct = ();

    type SerializeTupleVariant = ();

    type SerializeMap = MapSerializer;

    type SerializeStruct = ();

    type SerializeStructVariant = ();

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        let v = if v {
            "true"
        } else {
            "false"
        };
        self.result = MText(v);
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        todo!()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok> where T: Serialize {
        todo!()
    }

    fn serialize_unit(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_struct(self, name: &'static str) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> std::result::Result<Self::Ok, Self::Error> where T: Serialize {
        todo!()
    }

    fn serialize_newtype_variant<T: ?Sized>(self, name: &'static str, variant_index: u32, variant: &'static str, value: &T) -> std::result::Result<Self::Ok, Self::Error> where T: Serialize {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> std::result::Result<Self::SerializeSeq, Self::Error> {
        todo!()
    }

    fn serialize_tuple(self, len: usize) -> std::result::Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> std::result::Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> std::result::Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> std::result::Result<Self::SerializeMap, Self::Error> {
        todo!()
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> std::result::Result<Self::SerializeStruct, Self::Error> {
        todo!()
    }

    fn serialize_struct_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> std::result::Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }
}

pub struct TupleSerializer {

}

impl SerializeTuple for TupleSerializer {

    type Ok = ();

    type Error = ();

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> where T: Serialize {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

}

pub struct MapSerializer<'a> {
    serializer: &'a mut StructureSerializer,
}

impl SerializeMap for MapSerializer {

    type Ok = ();

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error> where T: Serialize {

        self.serializer.result.push(key)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> where T: Serialize {
        value.serialize(self.serializer)?;
        self.
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(&mut self, key: &K, value: &V) -> Result<(), Self::Error> where K: Serialize, V: Serialize {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.serializer.result.push('}');
        Ok(Self::Ok)
    }

}


