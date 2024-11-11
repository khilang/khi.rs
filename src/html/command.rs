//! Command binary for the XML/HTML-preprocessor.
//!
//! Test: cargo run --bin khi-html-cmd --features="html" -- examples/frontpage.html.khi
//! Test: cargo run --bin khi-html-cmd --features="html" -- examples/fruits.xml.khi

use std::env;
use std::fs::File;
use std::io::Read;
use khi::html::{PreprocessorError, write_html};
use khi::parse::{parse_value_str};
use khi::parse::parser::error_to_string;

fn main() {
    match preprocess() {
        Ok(output) => print!("{}\n\n", output),
        Err(error) => eprint!("{}\n\n", error),
    };
}

fn preprocess() -> Result<String, String> {
    let mut args = env::args();
    args.next(); // The first arg is the binary. Skip.
    if let Some(first) = args.next() {
        let mut file = File::open(first).unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        eprint!("Preprocessing document of size: {}\n\n", source.len());
        let document = match parse_value_str(&source) {
            Ok(document) => document,
            Err(errors) => {
                let mut errs = String::new();
                for e in errors {
                    errs.push_str(&error_to_string(&e));
                    errs.push('\n');
                }
                return Err(errs);
            },
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
                    PreprocessorError::IllegalTuple(at) => {
                        format!("Illegal tuple at {}:{}.", at.line, at.column)
                    }
                })
            }
        }
    } else {
        Err(format!("Specify source file as first argument."))
    }
}
