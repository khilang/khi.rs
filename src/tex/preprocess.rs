// '%' must be inserted as "\%". % indicates a TeX comment.
// '$' => '\$'
// '^' must be inserted as "\^". ^ is the superscript operator in math mode, and reserved in text mode.
// '_' must be inserted as "\_". _ is the subscript operator in math mode, and reserved in text mode.
// '&' must be inserted as "\&". & is the tabulation operator.
// '#' must be inserted as "\#". # is the argument substitution operator.
// '\' must be inserted as "\textbackslash" in text and "\backslash" or "\setminus" in math. "\\" indicates a line break.

use std::ops::Deref;
use crate::lex::Position;
use crate::parse::{ParsedValue, ParsedPattern, ParsedTable};
use crate::{Pattern, Table, Text, Element};
use crate::tex::preprocess::PreprocessorError::{IllegalDictionary, IllegalTable};

pub fn write_tex(structure: &ParsedValue) -> Result<String, PreprocessorError> {
    let mut output = String::new();
    let mut writer = Writer { output: &mut output, column: 1, newline: 60, last: LastType::Whitespace };
    writer.write_tex_inner(structure)?;
    Ok(output)
}

pub struct Writer<'a> {
    output: &'a mut String,
    column: usize,
    // 0 for never newline.
    newline: usize,
    last: LastType,
}

#[derive(Eq, PartialEq)]
enum LastType {
    Whitespace,
    Glyph,
    Caret,
    Underscore,
}

impl <'a> Writer<'a> {

    fn push(&mut self, char: char) {
        if char.is_whitespace() {
            if self.last == LastType::Caret || self.last == LastType::Underscore {

            } else {
                if self.last != LastType::Whitespace {
                    if self.newline == 0 || self.column < self.newline {
                        self.output.push(' ');
                    } else {
                        self.output.push('\n');
                        self.column = 1;
                    };
                    self.last = LastType::Whitespace;
                    self.column += 1;
                };
            }
        } else if char == '^' {
            self.output.push('^');
            self.last = LastType::Caret;
            self.column += 1;
        } else if char == '_' {
            self.output.push('_');
            self.last = LastType::Underscore;
            self.column += 1;
        } else {
            self.output.push(char);
            self.last = LastType::Glyph;
            self.column += 1;
        };
    }

    fn push_str(&mut self, str: &str) {
        for c in str.chars() {
            if c == '$' {
                self.output.push('\\');
                self.output.push('$');
                self.last = LastType::Glyph;
                self.column += 2;
            } else if c == '%' {
                self.output.push('\\');
                self.output.push('%');
                self.last = LastType::Glyph;
                self.column += 2;
            } else if c == '&' {
                self.output.push('\\');
                self.output.push('&');
                self.last = LastType::Glyph;
                self.column += 2;
            } else {
                self.push(c);
            }
        }
    }

    fn write_tex_inner(&mut self, expression: &ParsedValue) -> Result<(), PreprocessorError> {
        for component in expression.iter_components_with_whitespace() {
            match component {
                Element::Substance(ParsedValue::Nil(..)) => {
                    self.push('{');
                    self.push('}');
                }
                Element::Substance(ParsedValue::Text(text, ..)) => {
                    if self.last == LastType::Caret || self.last == LastType::Underscore {
                        self.push('{');
                        self.push_str(text.as_str());
                        self.push('}');
                    } else {
                        self.push_str(text.as_str());
                    }

                }
                Element::Substance(ParsedValue::Table(table, from, to)) => {
                    self.write_tabulation(&table, *from)?;
                }
                Element::Substance(ParsedValue::Dictionary(dictionary, from, to)) => {
                    return Err(IllegalDictionary(*from));
                }
                Element::Substance(ParsedValue::Pattern(command, from, to)) => {
                    self.write_macro(&command, *from)?;
                }
                Element::Substance(compound) => {
                    if self.last == LastType::Caret || self.last == LastType::Underscore {
                        self.push('{');
                        self.write_tex_inner(compound.as_expression())?;
                        self.push('}');
                    } else {
                        self.write_tex_inner(compound.as_expression())?;
                    }

                }
                Element::Whitespace => {
                    self.push(' ');
                }
            }
        };
        Ok(())
    }

    fn write_tabulation(&mut self, table: &ParsedTable, at: Position) -> Result<(), PreprocessorError> {
        if table.columns == 0 {
            return Err(PreprocessorError::ZeroTable(at))
        };
        for row in table.iter_rows() {
            let mut columns = row.iter();
            if let Some(c) = columns.next() {
                self.write_tex_inner(c)?;
            };
            while let Some(c) = columns.next() {
                self.push('&');
                self.write_tex_inner(c)?;
            };
            self.push('\\');
            self.push('\\');
        };
        Ok(())
    }

    fn write_macro(&mut self, directive: &ParsedPattern, at: Position) -> Result<(), PreprocessorError> {
        let mut command = directive.name.deref();
        if command.eq("$") {
            self.push('$');
            let structure = directive.arguments.get(0).unwrap().as_composite();
            self.write_tex_inner(structure)?;
            self.push('$');
        } else if command.eq("diag!") {

        } else if command.eq("p") {
            self.push_str("\n\n");
        } else if command.eq("n") {
            self.push_str("\\\\");
        } else if command.eq("\\") {
            self.push_str("\n\\\\");
        } else if command.eq("def!") {
            if directive.length() != 3 {
                panic!()
            }
            let defined = directive.get(0).unwrap().as_directive().unwrap();
            let args = directive.get(1).unwrap().as_composite();
            let arg = directive.get(2).unwrap().as_composite();
            self.output.push_str("\\newcommand");
            self.output.push('{');
            self.output.push('\\');
            self.output.push_str(defined.name.deref());
            self.output.push('}');
            self.output.push('[');
            self.write_tex_inner(args)?;
            self.output.push(']');
            self.output.push('{');
            self.write_tex_inner(arg)?;
            self.output.push('}');
        } else {
            // Regular command.
            let mut arguments = directive.iter_arguments();
            if command.ends_with("'") {
                command = &command[0..command.len() - 1];
                self.push('\\');
                self.push_str(command);
                if let Some(argument) = arguments.next() {
                    match argument {
                        ParsedValue::Nil(_, _) => {
                            self.push_str("[]");
                        }
                        ParsedValue::Text(text, ..) => {
                            self.push('[');
                            self.push_str(&text.as_str());
                            self.push(']');
                        }
                        ParsedValue::Table(table, from, to) => {
                            return Err(IllegalTable(*from));
                        }
                        ParsedValue::Dictionary(dictionary, from, to) => {
                            return Err(IllegalDictionary(*from));
                        }
                        ParsedValue::Pattern(directive, from, to) => {
                            self.push('{');
                            self.write_macro(directive, *from)?;
                            self.push('}');
                        }
                        compound => {
                            self.push('{');
                            self.write_tex_inner(compound.as_composite())?;
                            self.push('}');
                        }
                    }
                } else {
                    return Err(PreprocessorError::MissingOptionalArgument(at))
                }
            } else {
                self.push('\\');
                self.push_str(command);
            }
            while let Some(argument) = arguments.next() {
                match argument {
                    ParsedValue::Nil(..) => {
                        self.push_str("{}");
                    }
                    ParsedValue::Text(text, ..) => {
                        self.push('{');
                        self.push_str(&text.str.deref());
                        self.push('}');
                    }
                    ParsedValue::Table(sequence, from, to) => {
                        return Err(IllegalTable(*from));
                    }
                    ParsedValue::Dictionary(dictionary, from, to) => {
                        return Err(IllegalDictionary(*from));
                    }
                    ParsedValue::Pattern(command, from, to) => {
                        self.push('{');
                        self.write_macro(command, *from)?;
                        self.push('}');
                    }
                    compound => {
                        self.push('{');
                        self.write_tex_inner(compound.as_composite())?;
                        self.push('}');
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
    ZeroTable(Position),
    UnknownCommand(Position, String),
    MissingOptionalArgument(Position),
}
