// '%' must be inserted as "\%". % indicates a TeX comment.
// '$' => '\$'
// '^' must be inserted as "\^". ^ is the superscript operator in math mode, and reserved in text mode.
// '_' must be inserted as "\_". _ is the subscript operator in math mode, and reserved in text mode.
// '&' must be inserted as "\&". & is the tabulation operator.
// '#' must be inserted as "\#". # is the argument substitution operator.
// '\' must be inserted as "\textbackslash" in text and "\backslash" or "\setminus" in math. "\\" indicates a line break.

use std::ops::Deref;
use crate::lex::Position;
use crate::parse::{ParsedComponent, ParsedDirective, ParsedExpression, ParsedTable};
use crate::{Component, Directive, Expression, Table, Text, WhitespaceOption};
use crate::tex::preprocess::PreprocessorError::{IllegalDictionary, IllegalTable};

pub fn write_tex(structure: &ParsedExpression) -> Result<String, PreprocessorError> {
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

    fn write_tex_inner(&mut self, expression: &ParsedExpression) -> Result<(), PreprocessorError> {
        for component in expression.iter_components_with_whitespace() {
            match component {
                WhitespaceOption::Component(ParsedComponent::Empty(..)) => {
                    self.push('{');
                    self.push('}');
                }
                WhitespaceOption::Component(ParsedComponent::Text(text)) => {
                    if self.last == LastType::Caret || self.last == LastType::Underscore {
                        self.push('{');
                        self.push_str(text.as_str());
                        self.push('}');
                    } else {
                        self.push_str(text.as_str());
                    }

                }
                WhitespaceOption::Component(ParsedComponent::Table(table)) => {
                    self.write_tabulation(&table)?;
                }
                WhitespaceOption::Component(ParsedComponent::Dictionary(dictionary)) => {
                    return Err(IllegalDictionary(dictionary.from));
                }
                WhitespaceOption::Component(ParsedComponent::Directive(command)) => {
                    self.write_macro(&command)?;
                }
                WhitespaceOption::Component(compound) => {
                    if self.last == LastType::Caret || self.last == LastType::Underscore {
                        self.push('{');
                        self.write_tex_inner(compound.as_expression())?;
                        self.push('}');
                    } else {
                        self.write_tex_inner(compound.as_expression())?;
                    }

                }
                WhitespaceOption::Whitespace => {
                    self.push(' ');
                }
            }
        };
        Ok(())
    }

    fn write_tabulation(&mut self, table: &ParsedTable) -> Result<(), PreprocessorError> {
        if table.columns == 0 {
            return Err(PreprocessorError::ZeroTable(table.from))
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

    fn write_macro(&mut self, directive: &ParsedDirective) -> Result<(), PreprocessorError> {
        let command = &directive.directive;
        if command.deref().eq("$") {
            self.push('$');
            let structure = directive.arguments.get(0).unwrap().as_expression();
            self.write_tex_inner(structure)?;
            self.push('$');
        } else if command.deref().eq("@diag") {

        } else if command.ends_with('!') {
            return Err(PreprocessorError::UnknownCommand(directive.from, String::from(command.deref().deref())));
        } else if command.deref().eq("p") {
            self.push_str("\n\n");
        } else if command.deref().eq("n") {
            self.push_str("\\\\");
        } else if command.deref().eq("\\") {
            self.push_str("\n\\\\");
        } else if command.deref().eq("@def") {
            if directive.length() != 3 {
                panic!()
            }
            let defined = directive.get(0).unwrap().as_directive().unwrap();
            let args = directive.get(1).unwrap().as_expression();
            let arg = directive.get(2).unwrap().as_expression();
            self.output.push_str("\\newcommand");
            self.output.push('{');
            self.output.push('\\');
            self.output.push_str(defined.directive.deref());
            self.output.push('}');
            self.output.push('[');
            self.write_tex_inner(args)?;
            self.output.push(']');
            self.output.push('{');
            self.write_tex_inner(arg)?;
            self.output.push('}');
        } else {
            // Regular command.
            self.push('\\');
            self.push_str(command);
            let mut arguments = directive.iter_arguments();
            if command.ends_with("'") {
                if let Some(argument) = arguments.next() {
                    match argument {
                        ParsedComponent::Empty(_, _) => {
                            self.push_str("[]");
                        }
                        ParsedComponent::Text(text) => {
                            self.push('[');
                            self.push_str(&text.as_str());
                            self.push(']');
                        }
                        ParsedComponent::Table(table) => {
                            return Err(IllegalTable(table.from));
                        }
                        ParsedComponent::Dictionary(dictionary) => {
                            return Err(IllegalDictionary(dictionary.from));
                        }
                        ParsedComponent::Directive(directive) => {
                            self.push('{');
                            self.write_macro(directive)?;
                            self.push('}');
                        }
                        compound => {
                            self.push('{');
                            self.write_tex_inner(compound.as_expression())?;
                            self.push('}');
                        }
                    }
                } else {
                    return Err(PreprocessorError::MissingOptionalArgument(directive.from))
                }
            }
            while let Some(argument) = arguments.next() {
                match argument {
                    ParsedComponent::Empty(..) => {
                        self.push_str("{}");
                    }
                    ParsedComponent::Text(text) => {
                        self.push('{');
                        self.push_str(&text.str.deref());
                        self.push('}');
                    }
                    ParsedComponent::Table(sequence) => {
                        return Err(IllegalTable(sequence.from));
                    }
                    ParsedComponent::Dictionary(dictionary) => {
                        return Err(IllegalDictionary(dictionary.from));
                    }
                    ParsedComponent::Directive(command) => {
                        self.push('{');
                        self.write_macro(command)?;
                        self.push('}');
                    }
                    compound => {
                        self.push('{');
                        self.write_tex_inner(compound.as_expression())?;
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
    TabulateExpectedArg(Position),
    ZeroTable(Position),
    UnknownCommand(Position, String),
    MissingOptionalArgument(Position),
}
