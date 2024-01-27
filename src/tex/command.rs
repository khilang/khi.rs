//! Command binary for the TeX-preprocessor.

use std::env;
use std::fs::File;
use std::io::Read;
use khi::parse::{error_to_string, parse_expression_str};
use khi::tex::{PreprocessorError, write_tex};

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
        //print!("{}\n\n", parse);
        let output = match write_tex(&parse) {
            Ok(o) => o,
            Err(e) => match e {
                PreprocessorError::IllegalTable(at) => {
                    format!("Illegal sequence at {}:{}.", at.line, at.column)
                }
                PreprocessorError::IllegalDictionary(at) => {
                    format!("Illegal dictionary at {}:{}.", at.line, at.column)
                }
                PreprocessorError::ZeroTable(at) => {
                    format!("Table cannot be empty at {}:{}.", at.line, at.column)
                }
                PreprocessorError::UnknownCommand(at, directive) => {
                    format!("Unknown command {} at {}:{}.", &directive, at.line, at.column)
                }
                PreprocessorError::MissingOptionalArgument(at) => {
                    format!("Missing optional argument at {}:{}.", at.line, at.column)
                }
            },
        };
        Ok(output)
    } else {
        Err(format!("Specify source file as first argument."))
    };
}
