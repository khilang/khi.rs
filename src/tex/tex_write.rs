//! Generation of TeX from Dx.


// todo: insert reserved and special TeX characters correctly.
// '%' must be inserted as "\%". % indicates a TeX comment.
// '^' must be inserted as "\^". ^ is the superscript operator in math mode, and reserved in text mode.
// '_' must be inserted as "\_". _ is the subscript operator in math mode, and reserved in text mode.
// '&' must be inserted as "\&". & is the tabulation operator.
// '#' must be inserted as "\#". # is the argument substitution operator.
// '\' must be inserted as "\textbackslash" in text and "\backslash" or "\setminus" in math. "\\" indicates a line break.


use std::fmt::format;
use crate::parse::{ParsedArgument, ParsedDictionary, ParsedFunction, ParsedSymbol, ParsedGrouping, ParsedSequence, ParsedQuote, ParsedExpression, ParsedFunctionArgument};


pub fn write_tex(expression: &ParsedExpression) -> Result<String, String> {
    let mut output = String::new();
    write_tex_inner(expression, &mut output)?;
    Ok(output)
}


fn write_tex_inner(expression: &ParsedExpression, output: &mut String) -> Result<(), String> {
    let mut iter = expression.iter();
    let mut last_is_text = false; // If text was pushed last, as opposed to a grouping. Used to insert spaces at correct places.
    let mut next_is_optional = false;
    macro_rules! push_surround {
        ( $inner:expr ) => {
            {
                if next_is_optional {
                    output.push('[');
                    $inner;
                    output.push(']');
                    next_is_optional = false;
                } else {
                    output.push('{');
                    $inner;
                    output.push('}');
                };
            }
        }
    }
    loop {
        if let Some(argument) = iter.next() {
            match argument {
                ParsedArgument::Symbol(ParsedSymbol { symbol, .. }) => {
                    if last_is_text {
                        output.push(' ');
                    };
                    output.push_str(&symbol);
                    last_is_text = true;
                }
                ParsedArgument::Quote(ParsedQuote { quote: string, .. }) => {
                    if last_is_text {
                        output.push(' ');
                    };
                    output.push_str(&string);
                    last_is_text = true;
                }
                ParsedArgument::Function(ParsedFunction { arguments, from, .. }) => {
                    let mut function_iter = arguments.iter();
                    if let Some(command_name) = function_iter.next() {
                        match command_name {
                            ParsedFunctionArgument::Positional { argument } => {
                                match argument {
                                    ParsedArgument::Symbol(ParsedSymbol { symbol, from, .. }) => {
                                        if symbol.len() == 0 {
                                            return Err(format!("Preprocessor error: Empty macro at {}:{}.", from.line, from.column));
                                        };
                                        if symbol == "\\" {
                                            // Special command. todo: allow arbitrary special commands.
                                            output.push_str("\\\\");
                                        } else {
                                            // Regular text command. todo: check that only alphabetic characters are inserted.
                                            output.push('\\');
                                            if symbol.ends_with('?') {
                                                output.push_str(&symbol[0..symbol.len() - 1]);
                                                next_is_optional = true;
                                            } else {
                                                output.push_str(symbol);
                                            };
                                        };
                                    }
                                    ParsedArgument::Quote(_) => {
                                        return Err(format!("Preprocessor error: Illegal string in macro at {}:{}.", from.line, from.column));
                                    }
                                    ParsedArgument::Function(..) => {
                                        return Err(format!("Preprocessor error: Illegal macro in macro at {}:{}.", from.line, from.column));
                                    }
                                    ParsedArgument::Sequence(..) => {
                                        return Err(format!("Preprocessor error: Illegal sequence in macro at {}:{}.", from.line, from.column));
                                    }
                                    ParsedArgument::Dictionary(..) => {
                                        return Err(format!("Preprocessor error: Illegal dictionary in macro at {}:{}.", from.line, from.column));
                                    }
                                    ParsedArgument::Grouping(..) => {
                                        return Err(format!("Preprocessor error: Illegal grouping in macro at {}:{}.", from.line, from.column));
                                    }
                                };
                            }
                            _ => panic!(),
                        }
                    }  else {
                        return Err(format!("Preprocessor error: Command at {}:{} is empty.", from.line, from.column))
                    };
                    last_is_text = true;
                    loop {
                        if let Some(function_argument) = function_iter.next() {
                            match function_argument {
                                ParsedFunctionArgument::Positional { argument } => {
                                    match argument {
                                        ParsedArgument::Symbol(ParsedSymbol { symbol: glyphs, .. }) => {
                                            push_surround!{ output.push_str(glyphs) };
                                        }
                                        ParsedArgument::Quote(ParsedQuote { quote: string, .. }) => {
                                            push_surround!{ output.push_str(string) };
                                        }
                                        ParsedArgument::Function(..) => {
                                            return Err(format!("Preprocessor error: Illegal macro as macro argument at {}:{}.", from.line, from.column));
                                        }
                                        ParsedArgument::Sequence(..) => {
                                            return Err(format!("Preprocessor error: Illegal sequence as macro argument at {}:{}.", from.line, from.column));
                                        }
                                        ParsedArgument::Dictionary(..) => {
                                            return Err(format!("Preprocessor error: Illegal dictionary as macro argument at {}:{}.", from.line, from.column));
                                        }
                                        ParsedArgument::Grouping(ParsedGrouping { expression, .. }) => {
                                            push_surround!{ write_tex_inner(expression, output)? };
                                        }
                                    }
                                }
                                _ => panic!(),
                            }
                            last_is_text = false;
                        } else {
                            break;
                        };
                    };
                }
                ParsedArgument::Sequence(ParsedSequence { from: start, .. }) => {
                    return Err(format!("Preprocessor error: Sequence not allowed at {}:{}.", start.line, start.column));
                }
                ParsedArgument::Dictionary(ParsedDictionary { from, .. }) => {
                    return Err(format!("Preprocessor error: Dictionary not allowed at {}:{}.", from.line, from.column));
                }
                ParsedArgument::Grouping(ParsedGrouping { expression, .. }) => {
                    push_surround!{ write_tex_inner(&expression, output)? };
                    last_is_text = false;
                }
            }
        } else {
            return Ok(());
        }
    }
}
