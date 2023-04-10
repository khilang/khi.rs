//! A LaTeX preprocessor processing UDL input.


use std::env;
use std::fs::File;
use std::io::Read;
use udl::parse::{parse_expression_document, ParseError};
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
            Err(error) => return Err(match error {
                ParseError::EscapingEndOfStream => {
                    format!("Escaping EOS.")
                }
                ParseError::ExpectedClosingBracket(at) => {
                    format!("Expected bracket closing at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedSequenceClosing(at) => {
                    format!("Expected sequence closing at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedClosingParenthesis(at) => {
                    format!("Expected parenthesis closing at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedClosingSquare(at) => {
                    format!("Expected closing crotchet at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedOpeningParenthesis(at) => {
                    format!("Expected opening parenthesis at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedColonAfterGroupOperator(at) => {
                    format!("Expected colon after grouping operator at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedCommandAfterGroupOperator(at) => {
                    format!("Expected command after grouping operator at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedCommandClosing(at) => {
                    format!("Expected command closing at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedCommandArgument(at) => {
                    format!("Expected command argument at at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedCommandKey(at) => {
                    format!("Expected command key at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedAttributeArgument(at) => {
                    format!("Expected attribute value at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedEntrySeparator(at) => {
                    format!("Expected entry separator at {}:{}.", at.line, at.column)
                }
                ParseError::ExpectedEnd(at) => {
                    format!("Expected EOS at {}:{}.", at.line, at.column)
                }
                ParseError::CommentedBracket(at) => {
                    format!("Commented bracket not allowed at {}:{}.", at.line, at.column)
                }
                ParseError::UnclosedQuote(at) => {
                    format!("Unclosed quote at {}:{}.", at.line, at.column)
                }
            }),
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
