//! Command binary for the XML/HTML-preprocessor.

use std::env;
use std::fs::File;
use std::io::Read;
use khi::html::{PreprocessorError, write_html};
use khi::parse::{error_to_string, parse_expression_str};

fn main() {
    match preprocess() {
        Ok(s) => print!("{}\n\n", s),
        Err(e) => print!("{}\n\n", e),
    };
}

fn preprocess() -> Result<String, String> {
    let mut args = env::args();
    args.next(); // The first arg is the binary. Skip.
    return if let Some(first) = args.next() {
        let mut file = File::open(first).unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        print!("Preprocessing document of size: {}\n\n", source.len());
        let parse = match parse_expression_str(&source) {
            Ok(parse) => parse,
            Err(error) => return Err(error_to_string(&error)),
        };
        match write_html(&parse) {
            Ok(o) => Ok(o),
            Err(e) => {
                Err(match e {
                    PreprocessorError::IllegalTable(at) => {
                        format!("Illegal sequence at {}:{}.", at.line, at.column)
                    }
                    PreprocessorError::IllegalDictionary(at) => {
                        format!("Expected dictionary at {}:{}.", at.line, at.column)
                    }
                    PreprocessorError::Custom(error) => error,
                    PreprocessorError::TooManyArguments(at) => {
                        format!("Tag at {}:{} has more than one argument.", at.line, at.column)
                    }
                    PreprocessorError::IllegalAttributeValue(name, at) => {
                        format!("Attribute {} at {}:{} has illegal value.", name, at.line, at.column)
                    }
                })
            }
        }
    } else {
        Err(format!("Specify source file as first argument."))
    };
}
