//! Generation of HTML from UDL.


use std::process::Output;
use crate::ast::{ParsedArgument, ParsedCommand, ParsedDictionary, ParsedExpression, ParsedSequence, ParsedText};
use crate::lex::Position;


/// Process an expression into HTML.
pub fn write_html(expression: &ParsedExpression, indent: bool) -> Result<String, PreprocessorError> {
    let mut output = String::new();
    write_html_inner(&mut output, expression, indent, 0)?;
    Ok(output)
}


pub enum PreprocessorError {
    IllegalSequence(Position),
    IllegalDictionary(Position),
    Custom(String),
    TooManyTagArguments(Position),
    IllegalAttributeValue(String, Position),
}


fn write_html_inner(output: &mut String, expression: &ParsedExpression, indent: bool, level: u32) -> Result<(), PreprocessorError> {
    let mut add_whitespace = false;
    let mut iter = expression.iter();
    loop {
        if let Some((argument, whitespace)) = iter.next() {
            match argument {
                ParsedArgument::Empty(..) => {
                    //////////////////////////////////////add_whitespace = true;
                }
                ParsedArgument::Text(ParsedText { text, .. }) => {
                    output.push('\n');
                    push_indent(output, indent, level);
                    if add_whitespace {
                        output.push(' ');
                    };
                    output.push_str(&text);
                    add_whitespace = whitespace;
                }
                ParsedArgument::Sequence(ParsedSequence { from, .. }) => {
                    return Err(PreprocessorError::IllegalSequence(from.clone()));
                }
                ParsedArgument::Dictionary(ParsedDictionary { from, .. }) => {
                    return Err(PreprocessorError::IllegalDictionary(from.clone()));
                }
                ParsedArgument::Command(command) => {
                    if add_whitespace {
                        output.push(' ');
                    };
                    write_html_tag(output, &command, indent, level)?;
                    add_whitespace = whitespace;
                }
                compound => {
                    if add_whitespace {
                        output.push(' ');
                    };
                    write_html_inner(output, compound, indent, level + 1)?;
                    add_whitespace = whitespace;
                }
            }
        } else {
            return Ok(());
        }
    }
}


fn write_html_tag(output: &mut String, command: &ParsedCommand, indent: bool, level: u32) -> Result<(), PreprocessorError> {
    let mut attribute_iter = command.attributes.iter();
    let name = &command.command;
    // Macros // todo: set up arbitrary macro names.
    if name.starts_with('@') {
        return if name == "@doctype" {
            if command.attributes.is_empty() && command.arguments.is_empty() {
                output.push('\n');
                push_indent(output, indent, level);
                output.push_str("<!DOCTYPE html>");
                Ok(())
            } else {
                Err(PreprocessorError::Custom(format!("@doctype macro cannot have attributes nor arguments.")))
            }
        } else if name == "@title" {
            if command.attributes.is_empty() && command.arguments.is_empty() {
                output.push_str("Untitled document");
                Ok(())
            } else {
                Err(PreprocessorError::Custom(format!("@title macro cannot have attributes nor arguments.")))
            }
        } else {
            Err(PreprocessorError::Custom(format!("Unknown macro {}.", name)))
        };
    };
    // Tag
    let argument = if command.arguments.len() == 0 {
        None
    } else if command.arguments.len() == 1 {
        command.arguments.get(0)
    } else {
        return Err(PreprocessorError::TooManyTagArguments(command.from));
    };
    output.push('\n');
    push_indent(output, indent, level);
    output.push_str(&format!("<{}", name));
    loop {
        if let Some(attribute) = attribute_iter.next() {
            let key = &attribute.key;
            let value = &attribute.value;
            match value {
                ParsedExpression::Empty(_, _) => {
                    output.push(' ');
                    output.push_str(&key.text);
                }
                ParsedExpression::Text(value) => {
                    output.push(' ');
                    output.push_str(&key.text);
                    output.push_str("=\"");
                    output.push_str(&value.text);
                    output.push('"');
                }
                _ => return Err(PreprocessorError::IllegalAttributeValue(key.text.clone(), attribute.key.from)),
            };
        } else {
            break;
        };
    };
    output.push('>');
    if let Some(argument) = argument {
        write_html_inner(output, argument, indent, level + 1)?;
    } else {
        return Ok(());
    };
    output.push('\n');
    push_indent(output, indent, level);
    output.push_str("</");
    output.push_str(name);
    output.push('>');
    Ok(())
}

fn push_indent(output: &mut String, indent: bool, level: u32) {
    if !indent {
        return;
    };
    let mut i = 0;
    while i < level {
        output.push(' ');
        i += 1;
    }
}
