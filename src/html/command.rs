//! Command binary for the XML/HTML-preprocessor.
//!
//! Test: cargo run --bin khi-html-cmd --features="html" -- examples/frontpage.html.khi
//! Test: cargo run --bin khi-html-cmd --features="html" -- examples/fruits.xml.khi

use std::env;
use std::fs::File;
use std::io::Read;
use khi::html::{PreprocessorError, write_html};
use khi::parse::{error_to_string, parse_expression_str};

fn main() {
    match preprocess() {
        Ok(output) => print!("{}\n\n", output),
        Err(error) => print!("{}\n\n", error),
    };
}

fn preprocess() -> Result<String, String> {
    let mut args = env::args();
    args.next(); // The first arg is the binary. Skip.
    if let Some(first) = args.next() {
        let mut file = File::open(first).unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        print!("Preprocessing document of size: {}\n\n", source.len());
        let document = match parse_expression_str(&source) {
            Ok(document) => document,
            Err(error) => return Err(error_to_string(&error)),
        };
        match write_html(&document) {
            Ok(output) => Ok(output),
            Err(error) => {
                Err(match error {
                    PreprocessorError::IllegalTable(at) => {
                        format!("Illegal table at {}:{}.", at.line, at.column)
                    }
                    PreprocessorError::MacroError(error) => error,
                    PreprocessorError::TooManyArguments(at) => {
                        format!("Tag at {}:{} has more than one argument.", at.line, at.column)
                    }
                })
            }
        }
    } else {
        Err(format!("Specify source file as first argument."))
    }
}
