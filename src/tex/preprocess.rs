//! Generation of TeX from UDL.


// todo: insert reserved and special TeX characters correctly.
// '%' must be inserted as "\%". % indicates a TeX comment.
// '^' must be inserted as "\^". ^ is the superscript operator in math mode, and reserved in text mode.
// '_' must be inserted as "\_". _ is the subscript operator in math mode, and reserved in text mode.
// '&' must be inserted as "\&". & is the tabulation operator.
// '#' must be inserted as "\#". # is the argument substitution operator.
// '\' must be inserted as "\textbackslash" in text and "\backslash" or "\setminus" in math. "\\" indicates a line break.


use std::fmt::format;
use crate::ast::{ParsedArgument, ParsedDirective, ParsedCompound, ParsedDictionary, ParsedExpression, ParsedSequence};
use crate::lex::Position;
use crate::tex::preprocess::PreprocessorError::{IllegalDictionary, IllegalSequence};


pub fn write_tex(expression: &ParsedExpression) -> Result<String, PreprocessorError> {
    let mut output = String::new();
    write_tex_inner(expression, &mut output, 0)?;
    Ok(output)
}


fn write_tex_inner(expression: &ParsedExpression, output: &mut String, level: u32) -> Result<(), PreprocessorError> {
    let mut iter = expression.iter();
    //let mut last_is_text = false; // If text was pushed last, as opposed to a grouping. Used to insert spaces at correct places.
    loop {
        if let Some(argument) = iter.next() {
            let (argument, whitespace) = argument;
            match argument {
                ParsedArgument::Empty(..) => { }
                ParsedArgument::Text(text) => {
                    output.push('\n');
                    push_indent(output, level);
                    output.push_str(&text.text);
                }
                ParsedArgument::Sequence(sequence) => {
                    return Err(IllegalSequence(sequence.from));
                }
                ParsedArgument::Dictionary(dictionary) => {
                    return Err(IllegalDictionary(dictionary.from));
                }
                ParsedArgument::Command(command) => {
                    write_macro(command, output, level)?;
                }
                compound => {
                    write_tex_inner(compound, output, level)?;
                }
            }
        } else {
            return Ok(());
        }
    }
}


fn write_macro(command: &ParsedDirective, output: &mut String, level: u32) -> Result<(), PreprocessorError> {
    let name = &command.directive;
    if name.starts_with('@') {
        return Ok(());
    } else if name.eq("$") {
        output.push('$');
        let argument = command.arguments.get(0).unwrap();
        write_tex_inner(argument, output, level)?;
        output.push('$');
        return Ok(());
    } else if name.eq("@tabulate-sq") {
        let dim = command.arguments.get(0).unwrap();

    } else if name.eq("\\") {
        output.push_str("\n\\\\");
        return Ok(());
    }
    output.push('\n');
    push_indent(output, level);
    output.push('\\');
    output.push_str(name);
    let mut arguments = command.arguments.iter();
    let mut nextopt = false;
    loop {
        if let Some(argument) = arguments.next() {
            match argument {
                ParsedExpression::Empty(..) => {
                    if nextopt {
                        output.push_str("[]");
                        nextopt = false;
                    } else {
                        output.push_str("{}");
                    }
                }
                ParsedExpression::Text(text) => {
                    if text.text.eq("*") {
                        nextopt = true;
                        continue;
                    }
                    if nextopt {
                        output.push('[');
                        output.push_str(&text.text);
                        output.push(']');
                        nextopt = false;
                        continue;
                    } else {
                        output.push('{');
                        output.push_str(&text.text);
                        output.push('}');
                    }
                }
                ParsedExpression::Sequence(sequence) => {
                    return Err(IllegalSequence(sequence.from));
                }
                ParsedExpression::Dictionary(dictionary) => {
                    return Err(IllegalDictionary(dictionary.from));
                }
                ParsedExpression::Command(command) => {
                    output.push('{');
                    write_macro(command, output, level + 1)?;
                    output.push('}');
                }
                compound => {
                    output.push_str("{\n");
                    write_tex_inner(compound, output, level + 1)?;
                    output.push('\n');
                    push_indent(output, level);
                    output.push('}');
                }
            }
        } else {
            break;
        };
    };
    Ok(())
}


fn push_indent(output: &mut String, level: u32) {
    let mut i = 0;
    while i < level {
        output.push(' ');
        i += 1;
    }
}


pub enum LastState {
    Whitespace,
    Underscore,
    Caret,
}


pub enum PreprocessorError {
    IllegalSequence(Position),
    IllegalDictionary(Position),
}
