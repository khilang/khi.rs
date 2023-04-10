//! Parser for UDL (Universal Data Language).


extern crate core;


#[cfg(feature = "parse")]
pub mod ast;
#[cfg(feature = "parse")]
pub mod lex;
#[cfg(feature = "parse")]
pub mod parse;

#[cfg(feature = "common")]
pub mod common;

#[cfg(feature = "html")]
pub mod html;

#[cfg(feature = "tex")]
pub mod tex;
