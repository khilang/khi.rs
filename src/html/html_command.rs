//! A HTML preprocessor processing Dx input.


use std::env;
use std::fs::File;
use std::io::Read;
use dxrs::parse::{ErrorType, parse_expression, ParseError};
use dxrs::html::write_html;


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
        let parse = match parse_expression(&source) {
            Ok(parse) => parse,
            Err(error) => return Err(match error {
                ParseError { position, error } => match error {
                    ErrorType::ClosingMismatch => format!("Illegal closing at {}:{}.", position.line, position.column),
                    ErrorType::EscapingEndOfStream => format!("Escaping EOS at {}:{}.", position.line, position.column),
                    ErrorType::IllegalSemicolon => format!("Illegal delimiter at {}:{}.", position.line, position.column),
                    ErrorType::InvalidKey => format!("Invalid key at {}:{}.", position.line, position.column),
                    ErrorType::IllegalColon => format!("Illegal colon at {}:{}.", position.line, position.column),
                    ErrorType::OptionNotFinished => format!("Option not finished at {}:{}.", position.line, position.column),
                    ErrorType::ExpectedColon => format!("Expected colon at {}:{}.", position.line, position.column),
                }
            }),
        };
        print!("{}\n\n", parse);
        let output = write_html(&parse)?;
        Ok(output)
    } else {
        Err(format!("Specify source file as first argument."))
    };
}

