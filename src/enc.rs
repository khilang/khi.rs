//! Encoding and decoding of data structures.

use std::char::ParseCharError;
use std::num::{ParseFloatError, ParseIntError};
use std::str::{FromStr, ParseBoolError};
use crate::{Directive, Expression, Table, Text, Component};
use crate::model::SimpleStructure;
use crate::parse::{ParsedDirective, ParsedExpression, ParsedText};

/// Decoding of primitives from text.
pub trait DecodeTextExt: Text<_, _, _, _, _> {
    fn parse_bool(&self) -> Result<bool, ParseBoolError>;
    fn parse_u8(&self) -> Result<u8, ParseIntError>;
    fn parse_u16(&self) -> Result<u16, ParseIntError>;
    fn parse_u32(&self) -> Result<u32, ParseIntError>;
    fn parse_u64(&self) -> Result<u64, ParseIntError>;
    fn parse_u128(&self) -> Result<u128, ParseIntError>;
    fn parse_i8(&self) -> Result<i8, ParseIntError>;
    fn parse_i16(&self) -> Result<i16, ParseIntError>;
    fn parse_i32(&self) -> Result<i32, ParseIntError>;
    fn parse_i64(&self) -> Result<i64, ParseIntError>;
    fn parse_i128(&self) -> Result<i128, ParseIntError>;
    fn parse_f32(&self) -> Result<f32, ParseFloatError>;
    fn parse_f64(&self) -> Result<f64, ParseFloatError>;
    fn parse_char(&self) -> Result<char, ParseCharError>;
}

pub enum ParsedEnum {
    Struct,
    Sequence,
    Unit,
    Newtype,
}

///
pub trait KhiEncode<> {
    fn khi_encode<St: Expression<_, _, _, _, _>>(&self) -> SimpleStructure;
}

pub trait KhiDecode<> {

}

///
pub trait DecodeKhi: Expression<_, _, _, _, _> {
    fn decode_structure<St: Expression<_, _, _, _, _>>(&self) -> Self;
}

pub trait DecodeType {
    fn decode<St: Expression<>>(structure: St) -> Self;
}

/// Extension trait that adds the decode method to types that can be decoded.
pub trait DecodeStructure<T>: Expression<> {
    fn decode(&self) -> T;
}

impl <St: Expression<>, T: DecodeType> DecodeStructure<T> for St {

}


impl <T> DecodeType<T> for T {
    fn decode(structure: St) -> Self {
        todo!()
    }
}

impl DecodeKhi for bool {

    fn decode_structure<St: Expression<_, _, _, _, _>>(&self) -> Self {
        todo!()
    }

}

pub trait DecodeStructure: Expression<_, _, _, _, _> {
    fn decode_bool(&self) -> Result<bool, ParseBoolError>;
    fn decode_u8(&self) -> Result<u8, ParseIntError>;
    fn decode_u16(&self) -> Result<u16, ParseIntError>;
    fn decode_u32(&self) -> Result<u32, ParseIntError>;
    fn decode_u64(&self) -> Result<u64, ParseIntError>;
    fn decode_u128(&self) -> Result<u128, ParseIntError>;
    fn decode_i8(&self) -> Result<i8, ParseIntError>;
    fn decode_i16(&self) -> Result<i16, ParseIntError>;
    fn decode_i32(&self) -> Result<i32, ParseIntError>;
    fn decode_i64(&self) -> Result<i64, ParseIntError>;
    fn decode_i128(&self) -> Result<i128, ParseIntError>;
    fn decode_f32(&self) -> Result<f32, ParseFloatError>;
    fn decode_f64(&self) -> Result<f64, ParseFloatError>;
    fn decode_char(&self) -> Result<char, ParseCharError>;
    fn decode_enum(&self) -> Result<(&str, &Directive), E>;
    fn decode_struct(&self) -> Result<(&str, &Self), E>;
    fn decode_dictionary(&self) -> Result<>;
    fn decode_tuple(&self);
    fn decode_list(&self);

}

impl <S: Expression<_, _, _, _, _>> DecodeStructure for S {

    fn decode_bool(&self) -> Result<bool, ParseBoolError> {
        todo!()
    }

    fn decode_u8(&self) -> Result<u8, ParseIntError> {
        todo!()
    }

    fn decode_u16(&self) -> Result<u16, ParseIntError> {
        todo!()
    }

    fn decode_u32(&self) -> Result<u32, ParseIntError> {
        todo!()
    }

    fn decode_u64(&self) -> Result<u64, ParseIntError> {
        todo!()
    }

    fn decode_u128(&self) -> Result<u128, ParseIntError> {
        todo!()
    }

    fn decode_i8(&self) -> Result<i8, ParseIntError> {
        todo!()
    }

    fn decode_i16(&self) -> Result<i16, ParseIntError> {
        todo!()
    }

    fn decode_i32(&self) -> Result<i32, ParseIntError> {
        todo!()
    }

    fn decode_i64(&self) -> Result<i64, ParseIntError> {
        todo!()
    }

    fn decode_i128(&self) -> Result<i128, ParseIntError> {
        todo!()
    }

    fn decode_f32(&self) -> Result<f32, ParseFloatError> {
        todo!()
    }

    fn decode_f64(&self) -> Result<f64, ParseFloatError> {
        todo!()
    }

    fn decode_char(&self) -> Result<char, ParseCharError> {
        todo!()
    }

    fn decode_enum(&self) -> Result<(&str, &Directive), E> {
        todo!()
    }

    fn decode_struct(&self) -> Result<(&str, &Self), E> {
        todo!()
    }

    fn decode_dictionary(&self) -> Result {
        todo!()
    }

    fn decode_tuple(&self) {
        todo!()
    }

    fn decode_list(&self) {
        todo!()
    }

}

/// Encoding of data structures.
pub trait EncodeStructure: Expression<_, _, _, _, _> {
    fn encode_bool(value: bool);
    fn encode_u8(value: u8);
    fn encode_u16(value: u16);
    fn encode_u32(value: u32);
    fn encode_u64(value: u64);
    fn encode_u128(value: u128);
    fn encode_i8(value: i8);
    fn encode_i16(value: i16);
    fn encode_i32(value: i32);
    fn encode_i64(value: i64);
    fn encode_i128(value: i128);
    fn encode_f32(value: f32);
    fn encode_f64(value: f64);
    fn encode_char(value: char);
    fn encode_str(value: &str);
    fn encode_enum(value: &ParsedEnum) -> Result<Self, ()>;
}

/// Encoded numbers may contain whitespace in their representation. It is also optional
/// whether to use a point `.` or a comma `,`.
pub fn normalize_number_str(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars();
    loop {
        if let Some(c) = chars.next() {
            if c.is_whitespace() { // Remove whitespace
                continue;
            } else if c == ',' { // Convert , to .
                out.push('.');
            } else {
                out.push(c);
            };
        } else {
            break;
        };
    };
    out
}

impl <T: Text<_, _, _, _, _>> DecodeTextExt for T {

    fn parse_bool(&self) -> Result<bool, ParseBoolError> {
        let str = self.as_str();
        bool::from_str(str)
    }

    fn parse_u8(&self) -> Result<u8, ParseIntError> {
        let str = normalize_number_str(self.as_str());
        u8::from_str(&str)
    }

    fn parse_u16(&self) -> Result<u16, ParseIntError> {
        let str = normalize_number_str(self.as_str());
        u16::from_str(&str)
    }

    fn parse_u32(&self) -> Result<u32, ParseIntError> {
        let str = normalize_number_str(self.as_str());
        u32::from_str(&str)
    }

    fn parse_u64(&self) -> Result<u64, ParseIntError> {
        let str = normalize_number_str(self.as_str());
        u64::from_str(&str)
    }

    fn parse_u128(&self) -> Result<u128, ParseIntError> {
        let str = normalize_number_str(self.as_str());
        u128::from_str(&str)
    }

    fn parse_i8(&self) -> Result<i8, ParseIntError> {
        let str = normalize_number_str(self.as_str());
        i8::from_str(&str)
    }

    fn parse_i16(&self) -> Result<i16, ParseIntError> {
        let str = normalize_number_str(self.as_str());
        i16::from_str(&str)
    }

    fn parse_i32(&self) -> Result<i32, ParseIntError> {
        let str = normalize_number_str(self.as_str());
        i32::from_str(&str)
    }

    fn parse_i64(&self) -> Result<i64, ParseIntError> {
        let str = normalize_number_str(self.as_str());
        i64::from_str(&str)
    }

    fn parse_i128(&self) -> Result<i128, ParseIntError> {
        let str = normalize_number_str(self.as_str());
        i128::from_str(&str)
    }

    fn parse_f32(&self) -> Result<f32, ParseFloatError> {
        let str = normalize_number_str(self.as_str());
        f32::from_str(&str)
    }

    fn parse_f64(&self) -> Result<f64, ParseFloatError> {
        let str = normalize_number_str(self.as_str());
        f64::from_str(&str)
    }

    fn parse_char(&self) -> Result<char, ParseCharError> {
        let str = self.as_str();
        let mut chars = str.chars();
        if let Some(char) = chars.next() {
            if chars.next().is_none() {
                Ok(char)
            } else {
                Err(ParseCharError)
            }
        } else {
            Err(ParseCharError)
        }
    }

}

pub trait StructureCommonExt: Expression<_, _, _, _, _> { }

impl <E: Expression<_, _, _, _, _>> StructureCommonExt for E { }

