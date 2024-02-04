//! Command binary for the TeX-preprocessor.
//!
//! Test: cargo run --bin khi-tex-cmd --features="tex" -- examples/equations.tex.khi

use std::env;
use std::fmt::format;
use std::fs::File;
use std::io::{Read, Write};
use khi::parse::{error_to_string, parse_expression_str};
use khi::tex::{PreprocessorError, write_tex};

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
        let mut file = File::open(&first).unwrap();
        let mut source = String::new();
        file.read_to_string(&mut source).unwrap();
        print!("Preprocessing document of size: {}\n\n", source.len());
        let document = match parse_expression_str(&source) {
            Ok(document) => document,
            Err(error) => return Err(error_to_string(&error)),
        };
        match write_tex(&document) {
            Ok(output) => {
                if let Some(second) = args.next() {
                    if first.eq(&second) {
                        return Err(format!("Trying to overwrite source!"));
                    }
                    let mut out = File::create(&second).unwrap();
                    out.write_all(output.as_bytes()).unwrap();
                    Ok(format!("Successfully generated document."))
                } else {
                    Ok(output)
                }
            },
            Err(error) => match error {
                PreprocessorError::IllegalTable(at) => {
                    Err(format!("Illegal sequence at {}:{}.", at.line, at.column))
                }
                PreprocessorError::IllegalDictionary(at) => {
                    Err(format!("Illegal dictionary at {}:{}.", at.line, at.column))
                }
                PreprocessorError::ZeroTable(at) => {
                    Err(format!("Table cannot be empty at {}:{}.", at.line, at.column))
                }
                PreprocessorError::MacroError(at, directive) => {
                    Err(format!("Unknown command {} at {}:{}.", &directive, at.line, at.column))
                }
                PreprocessorError::MissingOptionalArgument(at) => {
                    Err(format!("Missing optional argument at {}:{}.", at.line, at.column))
                }
            },
        }
    } else {
        Err(format!("Specify source file as first argument."))
    }
}
