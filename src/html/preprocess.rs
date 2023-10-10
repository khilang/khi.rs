use std::ops::Deref;
use crate::{Component, Dictionary, Directive, Expression, Text, WhitespaceOption};
use crate::lex::Position;
use crate::parse::{ParsedComponent, ParsedDictionary, ParsedDirective, ParsedExpression};

pub fn write_html(expression: &ParsedExpression) -> Result<String, PreprocessorError> {
    let mut output = String::new();
    let mut writer = XmlWriter { output: &mut output, column: 1, newline: 60, last: LastType::Whitespace, };
    writer.write_inner(expression)?;
    Ok(output)
}

pub struct XmlWriter<'a> {
    output: &'a mut String,
    column: usize,
    // 0 for never newline.
    newline: usize,
    last: LastType,
}

#[derive(Eq, PartialEq)]
enum LastType {
    Glyph,
    Whitespace,
}

impl XmlWriter<'_> {

    fn push_whitespace(&mut self) {
        if self.last != LastType::Whitespace {
            if self.newline != 0 && self.column > self.newline {
                self.output.push('\n');
                self.column = 1;
            } else {
                self.output.push(' ');
                self.column += 1;
            }
            self.last = LastType::Whitespace;
        }
    }

    fn push_str(&mut self, str: &str) {
        for c in str.chars() {
            if c.is_whitespace() {
                self.push_whitespace();
            } else {
                self.column += 1;
                self.output.push(c);
                self.last = LastType::Glyph;
            }
        }
    }

    fn push_non_breaking(&mut self, char: char) {
        self.column += 1;
        self.output.push(char);
        if char.is_whitespace() {
            self.last = LastType::Whitespace;
        } else {
            self.last = LastType::Glyph;
        }
    }

    fn push_str_non_breaking(&mut self, str: &str) {
        for c in str.chars() {
            self.push_non_breaking(c);
        }
    }

    fn write_inner(&mut self, expression: &ParsedExpression) -> Result<(), PreprocessorError> {
        for component in expression.iter_components_with_whitespace() {
            match component {
                WhitespaceOption::Component(ParsedComponent::Empty(..)) => {}
                WhitespaceOption::Component(ParsedComponent::Text(text)) => {
                    self.push_str(text.as_str());
                }
                WhitespaceOption::Component(ParsedComponent::Dictionary(dictionary)) => {
                    self.write_dictionary(dictionary)?;
                }
                WhitespaceOption::Component(ParsedComponent::Table(table)) => {
                    return Err(PreprocessorError::IllegalSequence(table.from));
                }
                WhitespaceOption::Component(ParsedComponent::Directive(directive)) => {
                    self.write_tag(directive)?;
                }
                WhitespaceOption::Component(compound) => {
                    self.write_inner(compound.as_expression())?;
                }
                WhitespaceOption::Whitespace => {
                    self.push_whitespace();
                }
            }
        }
        Ok(())
    }

    fn write_tag(&mut self, command: &ParsedDirective) -> Result<(), PreprocessorError> {
        let name = &command.directive;
        // Macro // todo: set up arbitrary macro names.
        if name.starts_with('@') {
            return if name.deref() == "@doctype" {
                if !command.attributes.is_empty() || command.arguments.len() != 1 {
                    return Err(PreprocessorError::Custom(format!("@doctype macro cannot have attributes and must have 1 argument.")))
                }
                let arg = command.arguments.get(0).unwrap();
                self.push_str_non_breaking("<!DOCTYPE ");
                self.push_str_non_breaking(arg.as_text().unwrap().as_str());
                self.push_str_non_breaking(">");
                Ok(())
            } else if name.deref() == "@raw" {
                if let Some(arg) = command.get(0) {
                    if let Some(text) = arg.as_text() {
                        self.output.push_str(text.as_str());
                        Ok(())
                    } else {
                        Err(PreprocessorError::Custom(format!("@raw can only take a text argument.")))
                    }
                } else {
                    Err(PreprocessorError::Custom(format!("@raw must have one text argument.")))
                }
            } else {
                Err(PreprocessorError::Custom(format!("Unknown macro {}.", name)))
            };
        };
        // Tag
        let argument = if command.arguments.len() == 0 {
            None // Self closing tag
        } else if command.arguments.len() == 1 {
            command.arguments.get(0) // Regular tag
        } else {
            return Err(PreprocessorError::TooManyTagArguments(command.from));
        };
        self.push_non_breaking('<');
        self.push_str_non_breaking(&name);
        for attribute in command.iter_attributes() {
            let key = attribute.0;
            let value = attribute.1;
            match value.as_component() {
                ParsedComponent::Empty(..) => {
                    self.push_non_breaking(' ');
                    self.push_str_non_breaking(key);
                }
                ParsedComponent::Text(value) => {
                    self.push_non_breaking(' ');
                    self.push_str_non_breaking(key);
                    self.push_str_non_breaking("=\"");
                    self.push_str_non_breaking(value.as_str());
                    self.push_non_breaking('"');
                }
                _ => return Err(PreprocessorError::IllegalAttributeValue(String::from(key), value.as_component().from())),
            };
        }
        self.push_non_breaking('>');
        if let Some(argument) = argument {
            self.write_inner(argument.as_expression())?;
        } else {
            return Ok(());
        };
        self.push_str_non_breaking("</");
        self.push_str_non_breaking(name);
        self.push_non_breaking('>');
        Ok(())
    }

    fn write_dictionary(&mut self, dictionary: &ParsedDictionary) -> Result<(), PreprocessorError> {
        for (key, value) in dictionary.iter_entries() {
            self.push_non_breaking('<');
            self.push_str_non_breaking(key);
            self.push_non_breaking('>');
            self.write_inner(value)?;
            self.push_str_non_breaking("</");
            self.push_str_non_breaking(key);
            self.push_non_breaking('>');
        }
        Ok(())
    }

}

pub enum PreprocessorError {
    IllegalSequence(Position),
    IllegalDictionary(Position),
    Custom(String),
    TooManyTagArguments(Position),
    IllegalAttributeValue(String, Position),
}
