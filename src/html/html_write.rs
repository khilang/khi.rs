//! Generation of HTML from Dx.


use std::slice::Iter;
use crate::parse::{ParsedArgument, ParsedExpression, ParsedFunction, ParsedFunctionArgument, ParsedGrouping, ParsedKey, ParsedQuote, ParsedSymbol};


pub fn write_html(expression: &ParsedExpression) -> Result<String, String> {
    let mut output = String::new();
    write_html_inner(&mut output, expression)?;
    Ok(output)
}


fn write_html_inner(output: &mut String, expression: &ParsedExpression) -> Result<(), String> {
    let mut last_text = false;
    let mut iter = expression.iter();
    loop {
        if let Some(argument) = iter.next() {
            match argument {
                ParsedArgument::Symbol(ParsedSymbol { symbol, .. }) => {
                    if last_text {
                        output.push(' ');
                    };
                    output.push_str(symbol);
                    last_text = true;
                }
                ParsedArgument::Quote(ParsedQuote { quote, .. }) => {
                    if last_text {
                        output.push(' ');
                    };
                    output.push_str(quote);
                    last_text = true;
                }
                ParsedArgument::Sequence(sequence) => {
                    return Err(format!("Preprocessor error: Illegal sequence at {}:{}.", sequence.from.line, sequence.from.column));
                }
                ParsedArgument::Dictionary(dictionary) => {
                    return Err(format!("Preprocessor error: Illegal dictionary at {}:{}.", dictionary.from.line, dictionary.from.column));
                }
                ParsedArgument::Grouping(grouping) => {
                    return Err(format!("Preprocessor error: Illegal grouping at {}:{}. A grouping must follow a tag function.", grouping.from.line, grouping.from.column));
                }
                ParsedArgument::Function(function) => {
                    write_html_tag(output, function, &mut iter)?;
                    last_text = false;
                }
            }
        } else {
            return Ok(());
        }
    }
}


fn write_html_tag(output: &mut String, function: &ParsedFunction, expression_iterator: &mut Iter<ParsedArgument>) -> Result<(), String> {
    let mut self_closing = false;
    let mut insert_whitespace = false;
    let mut function_iterator = function.arguments.iter();
    let name = if let Some(first) = function_iterator.next() {
        if let ParsedFunctionArgument::Positional { argument } = first {
            if let ParsedArgument::Symbol(ParsedSymbol { symbol, .. }) = argument {
                symbol
            } else {
                return Err(format!("Preprocessor error: The first argument in a tag must be a symbol at {}:{}.", function.from.line, function.from.column));
            }
        } else {
            return Err(format!("Preprocessor error: The first argument in a tag function cannot be an option or a flag in function at {}:{}.", function.from.line, function.from.column));
        }
    } else {
        return Err(format!("Preprocessor error: Tag is empty at {}:{}.", function.from.line, function.from.column));
    };
    if name.ends_with('!') { //todo: set up arbitrary macro names.
        if name == "doctype!" {
            output.push_str("<!DOCTYPE html>");
            return Ok(());
        } else if name == "title!" {
            output.push_str("Untitled document");
            return Ok(());
        };
    };
    output.push_str(&format!("<{}", name));
    // The remaining function arguments are attributes.
    loop {
        if let Some(argument) = function_iterator.next() {
            match argument {
                ParsedFunctionArgument::Positional { argument } => {
                    if let ParsedArgument::Symbol(ParsedSymbol { symbol, from, to }) = argument {
                        if symbol == "/" {
                            // If a slash is found, do not take a content grouping argument.
                            self_closing = true;
                        } else if symbol == "<>" {
                            insert_whitespace = true;
                        } else {
                            output.push(' ');
                            output.push_str(&symbol);
                        };
                    } else {
                        return Err(format!("Preprocessor error: Positional arguments must be symbols in function at {}:{}.", function.from.line, function.from.column));
                    };
                }
                ParsedFunctionArgument::Option { key: ParsedKey { key, .. }, value } => {
                    output.push(' ');
                    output.push_str(key);
                    output.push_str("=\"");
                    match value {
                        ParsedArgument::Symbol(ParsedSymbol { symbol, .. }) => {
                            output.push_str(&symbol);
                        }
                        ParsedArgument::Quote(ParsedQuote { quote, .. }) => {
                            output.push_str(&quote);
                        }
                        ParsedArgument::Sequence(..) | ParsedArgument::Dictionary(..) | ParsedArgument::Grouping(..) | ParsedArgument::Function(..) => {
                            return Err(format!("Preprocessor error: An option value can only be symbol or quote in tag function at {}:{}.", function.from.line, function.from.column));
                        }
                    };
                    output.push('"');
                }
                ParsedFunctionArgument::Flag { flag: ParsedKey { key, .. } } => {
                    output.push(' ');
                    output.push_str(key);
                }
            };
        } else {
            break;
        };
    };
    if self_closing {
        output.push_str(" />");
        return Ok(());
    };
    output.push('>');
    if insert_whitespace { //todo whitespace should be inserted before and after the element, not inside.
        output.push(' ');
    }
    // The next argument is the content of this tag.
    if let Some(argument) = expression_iterator.next() {
        match argument {
            ParsedArgument::Symbol(ParsedSymbol { symbol, .. }) => output.push_str(symbol),
            ParsedArgument::Quote(ParsedQuote { quote, .. }) => output.push_str(quote),
            ParsedArgument::Grouping(ParsedGrouping { expression, .. }) => write_html_inner(output, &expression)?,
            ParsedArgument::Function(function) => write_html_tag(output, function, expression_iterator)?,
            ParsedArgument::Sequence(..) | ParsedArgument::Dictionary(..) => {
                return Err(format!("Preprocessor error: Argument following function at {}:{} cannot be a sequence or dictionary.", function.from.line, function.from.column));
            }
        }
    } else {
        return Err(format!("Preprocessor error: Missing argument following function at {}:{}.", function.from.line, function.from.column));
    };
    // Closing tag.
    if insert_whitespace { //todo
        output.push(' ');
    }
    output.push_str("</");
    output.push_str(name);
    output.push('>');
    Ok(())
}

