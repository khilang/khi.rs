//! A LaTeX preprocessor processing UDL input.


use std::env;
use std::fs::File;
use std::io::Read;
use udl::parse::{error_to_string, parse_expression_document, ParseError};
use udl::tex::{PreprocessorError, write_tex};


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
        let parse = match parse_expression_document(&source) {
            Ok(parse) => parse,
            Err(error) => return Err(error_to_string(&error)),
        };
        print!("{}\n\n", parse);
        let output = match write_tex(&parse) {
            Ok(o) => o,
            Err(e) => match e {
                PreprocessorError::IllegalSequence(at) => {
                    format!("Illegal sequence at {}:{}.", at.line, at.column)
                }
                PreprocessorError::IllegalDictionary(at) => {
                    format!("Illegal dictionary at {}:{}.", at.line, at.column)
                }
            },
        };
        Ok(output)
    } else {
        Err(format!("Specify source file as first argument."))
    };
}
