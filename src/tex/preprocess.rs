// '%' must be inserted as "\%". % indicates a TeX comment.
// '$' => '\$'
// '^' must be inserted as "\^". ^ is the superscript operator in math mode, and reserved in text mode.
// '_' must be inserted as "\_". _ is the subscript operator in math mode, and reserved in text mode.
// '&' must be inserted as "\&". & is the tabulation operator.
// '#' must be inserted as "\#". # is the argument substitution operator.
// '\' must be inserted as "\textbackslash" in text and "\backslash" or "\setminus" in math. "\\" indicates a line break.

use std::fmt::Write;
use crate::lex::Position;
use crate::parse::{ParsedValue, ParsedTag, ParsedTable};
use crate::{Tag, Table, Text, Element, Compound, Value};

pub fn write_tex(structure: &ParsedValue) -> Result<String, PreprocessorError> {
    let mut output = String::new();
    let mut writer = Writer { output: &mut output, column: 1, break_mode: BreakMode::Mirror, last_type: LastType::Whitespace, line: 1 };
    writer.write_inner(structure)?;
    Ok(output)
}

pub struct Writer<'a> {
    output: &'a mut String,
    column: usize,
    break_mode: BreakMode,
    last_type: LastType,
    line: usize, // Last line read in the source file
}

enum BreakMode {
    Never, Margin(usize), Mirror
}

#[derive(Eq, PartialEq)]
enum LastType {
    Newline,
    Whitespace,
    Glyph,
    Caret,
    Underscore,
    Command,
}

impl Writer<'_> {

    fn push(&mut self, char: char) {
        if char.is_whitespace() {
            if self.last_type == LastType::Command {
                self.output.push('{');
                self.output.push('}');
                self.output.push(' ');
                self.last_type = LastType::Whitespace;
            } else if self.last_type == LastType::Caret || self.last_type == LastType::Underscore {
                //
            } else if self.last_type == LastType::Whitespace || self.last_type == LastType::Newline {
                //
            } else {
                self.output.push(' ');
                self.last_type = LastType::Whitespace;
                self.column += 1;
            }
        } else if char == '^' {
            self.contract_opportunity();
            self.output.push('^');
            self.last_type = LastType::Caret;
            self.column += 1;
        } else if char == '_' {
            self.contract_opportunity();
            self.output.push('_');
            self.last_type = LastType::Underscore;
            self.column += 1;
        } else {
            self.output.push(char);
            self.last_type = LastType::Glyph;
            self.column += 1;
        };
    }

    fn contract_opportunity(&mut self) {
        while self.output.ends_with(' ') {
            self.output.pop();
        }
    }

    fn normalize_and_push_str(&mut self, str: &str) {
        for c in str.chars() {
            if c == '$' {
                self.output.push('\\');
                self.output.push('$');
                self.last_type = LastType::Glyph;
                self.column += 2;
            } else if c == '%' {
                self.output.push('\\');
                self.output.push('%');
                self.last_type = LastType::Glyph;
                self.column += 2;
            } else if c == '&' {
                self.output.push('\\');
                self.output.push('&');
                self.last_type = LastType::Glyph;
                self.column += 2;
            } else {
                self.push(c);
            }
        }
    }

    /// React to the position of a value.
    fn break_opportunity(&mut self, position: Position) {
        let at_line = position.line;
        match self.break_mode {
            BreakMode::Never => {}
            BreakMode::Margin(margin) => {
                if margin < self.column {
                    if !matches!(self.last_type, LastType::Newline) {
                        self.contract_opportunity();
                        self.output.push('\n');
                        self.line += 1;
                        self.last_type = LastType::Newline;
                        self.column = 1;
                    }
                }
            }
            BreakMode::Mirror => {
                if self.line < at_line {
                    if matches!(self.last_type, LastType::Newline) {
                        self.output.push_str("%\n");
                    } else {
                        self.contract_opportunity();
                        self.output.push('\n');
                    }
                    self.line += 1;
                    self.last_type = LastType::Newline;
                    self.column = 1;
                    while self.line < at_line {
                        self.output.push_str("%\n");
                        self.line += 1;
                    }
                }
            }
        }
    }

    /// If an empty command was last written, insert a space.
    fn separate_command_opportunity(&mut self) {
        if self.last_type == LastType::Command {
            self.output.push(' ');
            self.last_type = LastType::Whitespace;
            self.column += 1;
        }
    }

}

impl Writer<'_> {

    fn write_inner(&mut self, value: &ParsedValue) -> Result<(), PreprocessorError> {
        match value {
            ParsedValue::Nil(at, _) => {
                self.break_opportunity(*at);
                self.push('{');
                self.push('}');
            }
            ParsedValue::Text(text, at, _) => {
                self.break_opportunity(*at);
                self.normalize_and_push_str(text.as_str());
            }
            ParsedValue::Dictionary(_, at, _) => {
                return Err(PreprocessorError::IllegalDictionary(*at));
            }
            ParsedValue::Table(table, at, _) => {
                self.break_opportunity(*at);
                self.write_tabulation(table, *at)?;
            }
            ParsedValue::Compound(compound, at, _) => {
                self.break_opportunity(*at);
                for element in compound.iter() {
                    match element {
                        Element::Solid(solid) => {
                            match solid {
                                ParsedValue::Nil(at, _) => {
                                    self.break_opportunity(*at);
                                    self.push('{');
                                    self.push('}');
                                }
                                ParsedValue::Text(text, at, _) => {
                                    self.break_opportunity(*at);
                                    if self.last_type == LastType::Caret || self.last_type == LastType::Underscore {
                                        self.push('{');
                                        self.normalize_and_push_str(text.as_str());
                                        self.push('}');
                                    } else {
                                        self.separate_command_opportunity();
                                        self.normalize_and_push_str(text.as_str());
                                    }
                                }
                                ParsedValue::Dictionary(_, at, _) => {
                                    return Err(PreprocessorError::IllegalDictionary(*at));
                                }
                                ParsedValue::Table(table, at, _) => {
                                    self.break_opportunity(*at);
                                    self.write_tabulation(&table, *at)?;
                                }
                                ParsedValue::Compound(compound, at, _) => {
                                    self.break_opportunity(*at);
                                    self.push('{');
                                    self.write_inner(solid)?;
                                    self.push('}');
                                }
                                ParsedValue::Tuple(_, at, _) => {
                                    return Err(PreprocessorError::IllegalTuple(*at));
                                }
                                ParsedValue::Tag(tag, at, _) => {
                                    self.break_opportunity(*at);
                                    self.write_macro(tag, *at)?;
                                }
                            }
                        }
                        Element::Space => {
                            self.break_opportunity(*at);
                            self.push(' ');
                        }
                    }
                };
            }
            ParsedValue::Tuple(_, at, _) => {
                return Err(PreprocessorError::IllegalTuple(*at));
            }
            ParsedValue::Tag(tag, at, _) => {
                self.break_opportunity(*at);
                self.write_macro(tag, *at)?;
            }
        }
        Ok(())
    }

    fn write_tabulation(&mut self, table: &ParsedTable, at: Position) -> Result<(), PreprocessorError> {
        if table.columns == 0 {
            return Err(PreprocessorError::ZeroTable(at))
        };
        for row in table.iter_rows() {
            let mut columns = row.iter();
            if let Some(c) = columns.next() {
                self.write_inner(c)?;
            };
            while let Some(c) = columns.next() {
                self.push('&');
                self.write_inner(c)?;
            };
            self.push('\\');
            self.push('\\');
        };
        Ok(())
    }

    fn write_macro(&mut self, tag: &ParsedTag, at: Position) -> Result<(), PreprocessorError> {
        let mut name = tag.name();
        let arguments = tag.get().unfold();
        if name.ends_with("!") {
            if name.eq("def!") {
                if arguments.len() != 3 {
                    return Err(PreprocessorError::MacroError(at, format!("def! must take 3 arguments.")));
                }
                let tag = arguments.get(0).unwrap().as_tag().unwrap();
                let arity = arguments.get(1).unwrap();
                let substitute = arguments.get(2).unwrap();
                self.output.push_str("\\newcommand");
                self.output.push('\\');
                self.output.push_str(tag.name());
                self.output.push('[');
                self.write_inner(arity)?;
                self.output.push(']');
                self.output.push('{');
                self.write_inner(substitute)?;
                self.output.push('}');
                self.last_type = LastType::Glyph;
            } else if name.eq("raw!") {
                if arguments.len() != 1 {
                    return Err(PreprocessorError::MacroError(at, format!("raw! must take 1 text argument.")));
                }
                let text = arguments.get(0).unwrap().as_text().unwrap();
                self.output.write_str(text.as_str()).or(Err(PreprocessorError::MacroError(at, format!("Error on writing to output in macro at {}:{}.", at.line, at.column))))?;
            } else {
                return Err(PreprocessorError::MacroError(at, format!("Unknown macro {}.", name)));
            }
        } else if name.eq("$") {
            self.push('$');
            let structure = arguments.get(0).unwrap();
            self.write_inner(structure)?;
            self.push('$');
        } else if name.eq("p") {
            self.normalize_and_push_str("{\\par}");
        } else if name.eq("n") {
            self.normalize_and_push_str("\\\\");
        }  else {
            // Regular command.
            let mut iter = arguments.iter();
            if name.ends_with("'") {
                name = &name[0..name.len() - 1];
                self.push('\\');
                self.normalize_and_push_str(name);
                if let Some(argument) = iter.next() {
                    match argument {
                        ParsedValue::Nil(_, at) => {
                            self.break_opportunity(*at);
                            self.normalize_and_push_str("[]");
                        }
                        ParsedValue::Text(text, at, from) => {
                            self.break_opportunity(*at);
                            self.push('[');
                            self.normalize_and_push_str(&text.as_str());
                            self.push(']');
                        }
                        ParsedValue::Dictionary(dictionary, at, to) => {
                            return Err(PreprocessorError::IllegalDictionary(*at));
                        }
                        ParsedValue::Table(table, at, to) => {
                            return Err(PreprocessorError::IllegalTable(*at));
                        }
                        ParsedValue::Compound(compound, at, to) => {
                            self.break_opportunity(*at);
                            self.push('[');
                            self.write_inner(argument)?;
                            self.push(']');
                        }
                        ParsedValue::Tuple(_, at, _) => {
                            return Err(PreprocessorError::IllegalTuple(*at));
                        }
                        ParsedValue::Tag(tag, at, to) => {
                            self.break_opportunity(*at);
                            self.push('[');
                            self.write_macro(tag, *at)?;
                            self.push(']');
                        }
                    }
                } else {
                    return Err(PreprocessorError::MissingOptionalArgument(at))
                }
            } else {
                self.push('\\');
                self.normalize_and_push_str(name);
            }
            if arguments.is_empty() { // No arguments - if followed by whitespace, insert empty {} after due to LaTeX scanner consuming following whitespace.
                self.last_type = LastType::Command;
            }
            while let Some(argument) = iter.next() {
                match argument {
                    ParsedValue::Nil(at, _) => {
                        self.break_opportunity(*at);
                        self.normalize_and_push_str("{}");
                    }
                    ParsedValue::Text(text, at, _) => {
                        self.break_opportunity(*at);
                        self.push('{');
                        self.normalize_and_push_str(text.as_str());
                        self.push('}');
                    }
                    ParsedValue::Dictionary(dictionary, at, to) => {
                        return Err(PreprocessorError::IllegalDictionary(*at));
                    }
                    ParsedValue::Table(table, at, to) => {
                        return Err(PreprocessorError::IllegalTable(*at));
                    }
                    ParsedValue::Compound(compound, at, to) => {
                        self.break_opportunity(*at);
                        self.push('{');
                        self.write_inner(argument)?;
                        self.push('}');
                    }
                    ParsedValue::Tuple(_, at, _) => {
                        return Err(PreprocessorError::IllegalTable(*at));
                    }
                    ParsedValue::Tag(tag, at, to) => {
                        self.break_opportunity(*at);
                        self.write_macro(tag, *at)?;
                    }
                }
            }
        };
        Ok(())
    }

}

pub enum PreprocessorError {
    IllegalTable(Position),
    IllegalDictionary(Position),
    IllegalTuple(Position),
    ZeroTable(Position),
    MacroError(Position, String),
    MissingOptionalArgument(Position),
}
