//! Serde deserializer for the Khi data format.

use std::fmt::{Debug, Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;
use hex::FromHexError;
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, StdError, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::value::StrDeserializer;
use crate::{Component, Dictionary, Directive, Expression, Table, Text};
use crate::enc::{DecodeTextExt, normalize_number_str};
use crate::model::SimpleStructure;
use crate::parse::{parse_expression_str, parse_dictionary_str, parse_table_str, ParsedValue, ParseError};

/// Deserialize a Khi structure string.
pub fn from_structure_str<T: Deserialize>(str: &str) -> Result<T> {
    match parse_structure(str) {
        Ok(x) => from_structure(parse),
        Err(e) => Err(Error::ParseError(e)),
    }
}

/// Deserialize a Khi table string.
pub fn from_table_str<T: Deserialize>(str: &str) -> Result<T> {
    match parse_table(str) {
        Ok(x) => from_structure(parse),
        Err(e) => Err(),
    }
}

/// Deserialize a Khi dictionary string.
pub fn from_dictionary_str<T: Deserialize>(str: &str) -> Result<T> {
    match parse_nullable_dictionary(string) {
        Ok(x) => from_structure(parse),
        Err(e) => Err(),
    }
}

/// Deserialize a Khi structure.
pub fn from_structure<S: Expression<_, _, _, _, _>, T: Deserialize>(structure: &S) -> Result<T> {
    let des = StructureDeserializer::new(structure);
    T::deserialize(des)
}

/// An extension trait adding specific deserialization functions to Deserialize types.
trait KhiDeserializableData: Deserialize {
    fn deserialize_khi_structure<St: Expression<_, _, _, _, _>>(structure: &St) -> Result<Self>;
    fn deserialize_khi_structure_str(str: &str) -> Result<Self>;
    fn deserialize_khi_table_str(str: &str) -> Result<Self>;
    fn deserialize_khi_dictionary_str(str: &str) -> Result<Self>;
}

impl <T: Deserialize> KhiDeserializableData for T {

    fn deserialize_khi_structure<St: Expression<_, _, _, _, _>>(structure: &St) -> Result<Self> {
        let deserializer = StructureDeserializer::new(structure);
        Self::deserialize(deserializer)
    }

    fn deserialize_khi_structure_str(str: &str) -> Result<Self> {
        todo!()let g: SimpleStructure;

    }

    fn deserialize_khi_table_str(str: &str) -> Result<Self> {
        todo!()
    }

    fn deserialize_khi_dictionary_str(str: &str) -> Result<Self> {
        todo!()
    }

}

trait DeserializableKhiStructure: Expression<> {
    fn deserialize<T: Deserialize>(&self) -> Result<T>;
}

impl <St: Expression> DeserializableKhiStructure for St {

}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

/// Khi deserialization error.
pub enum Error<'a> {
    ParseError(ParseError),

    NotSelfDescribing,
    InvalidBoolValue,
    InvalidBoolStructure,

    InvalidUnitStructure,
    InvalidUnitStructStructure,

    SeqNotList,
    InvalidSeqStructure,

    InvalidMapStructure,
    InvalidStructStructure,
    InvalidStructNameComponent,
    StructNameMismatch,

    InvalidTupleStructStructure,

    InvalidEnumStructure,

    InvalidOptionStructure,
    InvalidOptionDirectiveLabel,
    InvalidOptionDirectiveAttributes,

    InvalidTupleStructure,
    SeqNotTuple,

    EnumNameMismatch,

    InvalidOptionDirectiveArguments,

    HexDecodeError,

    ParseIntError { structure: &'a ParsedExpression, error: ParseIntError },
    ParseFloatError { structure: &'a ParsedExpression, error: ParseFloatError },
    InvalidStructure { structure: &'a ParsedExpression, expected: &'a str },

}

impl Debug for Error {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {

        }
    }

}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl StdError for Error { }

impl serde::de::Error for Error {

    fn custom<T>(msg: T) -> Self where T: Display {
        todo!()
    }

}

pub struct StructureDeserializer<'a,
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Pt>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Pt>,
    Cm: Component<Vl, Tx, Dc, Tb, Cm, Pt>,
    Pt: Pattern<Vl, Tx, Dc, Tb, Cm, Pt>,
> {
    value: &'a Vl,
}

impl <'a,
    St: Expression<Tx, Tb, Dc, Dr, Cm>,
    Tx: Text<St, Tb, Dc, Dr, Cm>,
    Tb: Table<St, Tx, Dc, Dr, Cm>,
    Dc: Dictionary<St, Tx, Tb, Dr, Cm>,
    Dr: Directive<St, Tx, Tb, Dc, Cm>,
    Cm: Component<St, Tx, Tb, Dc, Dr>,
> StructureDeserializer<St, Tx, Tb, Dc, Dr, Cm> {

    pub fn new(structure: &'a St) -> Self {
        Self { value: structure }
    }

}

macro_rules! des_num {
    ($self:ident, $visitor:ident, $ty:ty, $visit:ident, $errvar:ident) => {
        {
            if let Some(text) = $self.structure.conform_text() {
                let text = normalize_number_str(text.as_str());
                match $ty::from_str(&text) {
                    Ok(v) => $visitor.$visit(v),
                    Err(e) => Err(Error::$errvar { structure: $self.structure, error: e }),
                }
            } else {
                Err(Error::InvalStructure { structure: $self.structure, expected: "Tx" })
            }
        }
    }
}

impl <'de,
    St: Expression<Tx, Tb, Dc, Dr, Cm>,
    Tx: Text<St, Tb, Dc, Dr, Cm>,
    Tb: Table<St, Tx, Dc, Dr, Cm>,
    Dc: Dictionary<St, Tx, Tb, Dr, Cm>,
    Dr: Directive<St, Tx, Tb, Dc, Cm>,
    Cm: Component<St, Tx, Tb, Dc, Dr>,
> Deserializer for StructureDeserializer<St, Tx, Tb, Dc, Dr, Cm> {

    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::NotSelfDescribing)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(text) = self.value.conform_text() {
            if text.eq("1") {
                visitor.visit_bool(true)
            } else if text.eq("0") {
                visitor.visit_bool(false)
            } else {
                Err(Error::InvalidBoolValue)
            }
        } else {
            Err(Error::InvalidBoolStructure)
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        des_num!(self, visitor, i8, visit_i8, ParseIntError)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        des_num!(self, visitor, i16, visit_i16, ParseIntError)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        des_num!(self, visitor, i32, visit_i32, ParseIntError)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        des_num!(self, visitor, i64, visit_i64, ParseIntError)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        des_num!(self, visitor, u8, visit_u8, ParseIntError)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        des_num!(self, visitor, u16, visit_u16, ParseIntError)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        des_num!(self, visitor, u32, visit_u32, ParseIntError)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        des_num!(self, visitor, u64, visit_u64, ParseIntError)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        des_num!(self, visitor, f32, visit_f32, ParseFloatError)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        des_num!(self, visitor, f64, visit_f64, ParseFloatError)
    }

    //todo +
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(char) = self.value.conform_text() {
            let char = char.as_str();
            let mut chars = char.chars();
            let char = if let Some(char) = chars.next() {
                if chars.next().is_some() {
                    return Err(Error::CharDecodeError);
                };
                char
            } else {
                return Err(Error::CharDecodeError);
            };
            visitor.visit_char(char)
        } else {
            Err(Error::InvalidF64Structure)
        }
    }

    //todo +
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(str) = self.value.conform_text() {
            let str = str.as_str();
            visitor.visit_str(str)
        } else {
            Err(Error::InvalidStructure)
        }
    }

    //todo +
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(string) = self.value.conform_text() {
            let string = String::from(string.as_str());
            visitor.visit_string(string)
        } else {
            Err(Error::InvalidStructure)
        }
    }

    //todo +
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(hex) = self.value.conform_text() {
            let hex = hex.as_str();
            let bytes = match hex::decode(hex) {
                Ok(b) => b,
                Err(_) => return Err(Error::HexDecodeError),
            };
            visitor.visit_bytes(&bytes)
        } else {
            Err(Error::InvalidF64Structure)
        }
    }

    //todo +
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(hex) = self.value.conform_text() {
            let hex = hex.as_str();
            let bytes = match hex::decode(hex) {
                Ok(b) => b,
                Err(_) => return Err(Error::HexDecodeError),
            };
            visitor.visit_byte_buf(bytes)
        } else {
            Err(Error::InvalidF64Structure)
        }
    }

    // `Some(a)` is encoded as `<?>:a` and `None` is encoded as `<?>`.
    //todo +
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(option) = self.value.conform_directive() {
            let label = &option.label;
            if !label.eq("?") {
                return Err(Error::InvalidOptionDirectiveLabel);
            };
            if option.has_attributes() {
                return Err(Error::InvalidOptionDirectiveAttributes);
            };
            let len = option.length();
            if len == 0 {
                visitor.visit_none()
            } else if len == 1 {
                let content = option.get(0).unwrap().as_structure();
                let deserializer = StructureDeserializer { value: content };
                visitor.visit_some(deserializer)
            } else {
                Err(Error::InvalidOptionDirectiveArguments)
            }
        } else {
            Err(Error::InvalidOptionStructure)
        }
    }

    // todo +
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if !self.value.is_empty() {
            return Err(Error::InvalidUnitStructure);
        };
        visitor.visit_unit()
    }

    //todo +
    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let len = self.value.length();
        if len == 0 {
            visitor.visit_unit()
        } else if len == 1 {
            if let Some(text) = self.value.conform_text() {
                let mName = text.as_str();
                if name.eq(mName) {
                    visitor.visit_unit()
                } else {
                    Err(Error::StructNameMismatch)
                }
            } else {
                Err(Error::InvalidUnitStructStructure)
            }
        } else {
            Err(Error::InvalidUnitStructStructure)
        }
    }

    //todo +
    fn deserialize_newtype_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        visitor.visit_newtype_struct(self)
    }

    //todo +
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(table) = self.value.conform_table() {
            if table.is_list() {
                visitor.visit_seq(TableAccess { iter: table.iter() })
            } else {
                Err(Error::SeqNotList)
            }
        } else {
            Err(Error::InvalidSeqStructure)
        }
    }

    //todo +
    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(table) = self.value.conform_table() {
            if table.is_tuple() {
                visitor.visit_seq(TableAccess { iter: table.iter() })
            } else {
                Err(Error::SeqNotTuple)
            }
        } else {
            Err(Error::InvalidTupleStructure)
        }
    }

    //todo +
    fn deserialize_tuple_struct<V>(self, name: &'static str, len: usize, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let len = self.value.length();
        if len == 1 {
            self.deserialize_tuple(visitor)
        } else if len == 2 {
            let c0 = self.value.get(0).unwrap();
            if let Some(c0) = c0.as_text() {
                let c0 = c0.as_str();
                if name.eq(c0) {
                    let c1 = self.value.get(1).unwrap().as_expression();
                    let deserializer = StructureDeserializer { value: c1 };
                    deserializer.deserialize_tuple(visitor)
                } else {
                    Err(Error::StructNameMismatch)
                }
            } else {
                Err(Error::InvalidTupleStructStructure)
            }
        } else {
            Err(Error::InvalidTupleStructStructure)
        }
    }

    //todo +
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(dictionary) = self.value.conform_dictionary() {
            visitor.visit_map(DictionaryAccess { iter: dictionary.iter(), current: None })
        } else {
            Err(Error::InvalidMapStructure)
        }
    }

    //todo +
    fn deserialize_struct<V>(self, name: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let len = self.value.length();
        if len == 1 {
            self.deserialize_map(visitor)
        } else if len == 2 {
            let c0 = self.value.get(0).unwrap();
            if let Some(c0) = c0.as_text() {
                let c0 = c0.as_str();
                if c0.eq(name) {
                    let c1 = self.value.get(1).unwrap().as_expression();
                    let deserializer = StructureDeserializer { value: c1 };
                    deserializer.deserialize_map(visitor)
                } else {
                    Err(Error::StructNameMismatch)
                }
            } else {
                Err(Error::InvalidStructStructure)
            }
        } else {
            Err(Error::InvalidStructStructure)
        }
    }

    //todo +
    fn deserialize_enum<V>(self, name: &'static str, variants: &'static [&'static str], visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        let len = self.value.length();
        let content;
        if len == 1 {
            content = self.value.get(0).unwrap();
        } else if len == 2 {
            content = self.value.get(1).unwrap();
            let found_name = self.value.get(0).unwrap();
            if let Some(found_name) = found_name.as_text() {
                let found_name = found_name.as_str();
                if !name.eq(found_name) {
                    return Err(Error::EnumNameMismatch);
                };
            } else {
                return Err(Error::InvalidEnumStructure);
            };
        } else {
            return Err(Error::InvalidEnumStructure);
        };
        if let Some(content) = content.as_directive() {
            if content.has_attributes() {
                return Err(Error::InvalidEnumStructure);
            };
            let variant = content.label();
            let content_len = content.length();
            if content_len == 0 {
                Ok(EnumAccess { variant, content: None })
            } else if content_len == 1 {
                let content = content.get_argument(0).unwrap().as_expression();
                Ok(EnumAccess { variant, content: Some(content) })
            } else {
                Err(Error::InvalidEnumStructure)
            }
        } else {
            Err(Error::InvalidEnumStructure)
        }
    }

    //todo +
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        Err(Error::NotSelfDescribing)
    }

    fn is_human_readable(&self) -> bool {
        true
    }

}

pub struct TableAccess<E, F> where E: Expression<_, _, _, _, _>, F: Iterator<Item=E> {
    iter: U,
}

impl <'de, E, F> SeqAccess for TableAccess<E, F> {

    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>> where T: DeserializeSeed<'de> {
        Ok(self.iter.next())
    }

}

pub struct DictionaryAccess<
    St: Expression<Tx, Tb, Dc, Dr, Cm>,
    Tx: Text<St, Tb, Dc, Dr, Cm>,
    Tb: Table<St, Tx, Dc, Dr, Cm>,
    Dc: Dictionary<St, Tx, Tb, Dr, Cm>,
    Dr: Directive<St, Tx, Tb, Dc, Cm>,
    Cm: Component<St, Tx, Tb, Dc, Dr>,
    It: Iterator<Item=(&str, &St)>,
> {
    iter: I,
    current: Option<E>,
}

impl <'de,
    St: Expression<Tx, Tb, Dc, Dr, Cm>,
    Tx: Text<St, Tb, Dc, Dr, Cm>,
    Tb: Table<St, Tx, Dc, Dr, Cm>,
    Dc: Dictionary<St, Tx, Tb, Dr, Cm>,
    Dr: Directive<St, Tx, Tb, Dc, Cm>,
    Cm: Component<St, Tx, Tb, Dc, Dr>,
    It: Iterator<Item=(&str, &St)>,
> MapAccess for DictionaryAccess<St, Tx, Tb, Dc, Dr, Cm, It> {

    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>> where K: DeserializeSeed<'de> {
        self.current = self.iter.next();
        if let Some((k, _)) = self.current {
            let deserializer = StrDeserializer::new(k);
            seed.deserialize(deserializer)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value> where V: DeserializeSeed<'de> {
        if let Some((_, v)) = self.current {
            let deserializer = StructureDeserializer { value: v };
            seed.deserialize(deserializer)
        } else {
            Ok(None)
        }
    }

    fn next_entry_seed<K, V>(&mut self, kseed: K, vseed: V) -> Result<Option<(K::Value, V::Value)>> where K: DeserializeSeed<'de>, V: DeserializeSeed<'de> {
        self.current = self.iter.next();
        if let Some((k, v)) = self.current {
            let kdeserializer = StrDeserializer::new(k);
            let vdeserializer = StructureDeserializer { value: v };
            Ok(Some((kseed.deserialize(kdeserializer), vseed.deserialize(vdeserializer))))
        } else {
            Ok(None)
        }
    }

}

pub struct EnumAccess<'a, S: Expression<_, _, _, _, _>> {
    variant: &'a str,
    content: Option<&'a S>,
}

impl <'de, S> serde::de::EnumAccess for EnumAccess<S> {

    type Error = Error;

    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)> where V: DeserializeSeed<'de> {
        let variant = StrDeserializer::new(self.variant);
        let variant = seed.deserialize( variant);
        Ok((variant, self))
    }

}

impl <'de, S: Expression<_, _, _, _, _>> serde::de::VariantAccess for EnumAccess<S> {

    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        if self.content.is_none() {
            Ok(())
        } else {
            Err(Error::InvalidEnumStructure)
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value> where T: DeserializeSeed<'de> {
        if let Some(inner) = self.content {
            let inner = StructureDeserializer { value: inner };
            seed.deserialize(inner)
        } else {
            Err(Error::InvalidEnumStructure)
        }
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(inner) = self.content {
            let inner = StructureDeserializer { value: inner };
            inner.deserialize_seq(visitor)
        } else {
            Err(Error::InvalidEnumStructure)
        }
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value> where V: Visitor<'de> {
        if let Some(inner) = self.content {
            let inner = StructureDeserializer { value: inner };
            inner.deserialize_seq(visitor)
        } else {
            Err(Error::InvalidEnumStructure)
        }
    }

}
