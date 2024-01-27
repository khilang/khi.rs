use std::ops::Deref;
use crate::{Dictionary, Pattern, Value, Text, Element, Entry, Attribute, Composition};
use crate::lex::Position;
use crate::parse::{ParsedComposition, ParsedDictionary, ParsedPattern, ParsedValue};

pub fn write_html(value: &ParsedValue) -> Result<String, PreprocessorError> {
    let mut output = String::new();
    let mut writer = XmlWriter { output: &mut output, column: 1, newline: 60, last: LastType::Whitespace, };
    writer.write_inner(value)?;
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

    fn write_inner(&mut self, value: &ParsedValue) -> Result<(), PreprocessorError> {
        match value {
            ParsedValue::Nil(..) => {}
            ParsedValue::Text(text, ..) => {
                self.push_str(text.as_str());
            }
            ParsedValue::Dictionary(dictionary, from, to) => {
                self.write_dictionary(dictionary, *from)?;
            }
            ParsedValue::Table(table, from, to) => {
                return Err(PreprocessorError::IllegalTable(*from));
            }
            ParsedValue::Pattern(directive, from, to) => {
                self.write_tag(directive, *from)?;
            }
            ParsedValue::Composition(composition, from, to) => {
                for element in composition.iter() {
                    if let Element::Substance(value) = element {
                        self.write_inner(value)?;
                    } else {
                        self.push_whitespace();
                    }
                }
            }
        }
        Ok(())
    }

    fn write_tag(&mut self, pattern: &ParsedPattern, at: Position) -> Result<(), PreprocessorError> {
        let name = &pattern.name;
        // Macro // todo: set up arbitrary macro names.
        if name.ends_with('!') {
            return if name.deref() == "doctype!" {
                if !pattern.attributes.is_empty() || pattern.arguments.len() != 1 {
                    return Err(PreprocessorError::Custom(format!("@doctype macro cannot have attributes and must have 1 argument.")))
                }
                let arg = pattern.arguments.get(0).unwrap();
                self.push_str_non_breaking("<!DOCTYPE ");
                self.push_str_non_breaking(arg.as_text().unwrap().as_str());
                self.push_str_non_breaking(">");
                Ok(())
            } else if name.deref() == "raw!" {
                if let Some(arg) = pattern.get(0) {
                    if let Some(text) = arg.as_text() {
                        self.output.push_str(text.get_str());
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
        let argument = if pattern.arguments.len() == 0 {
            None // Self closing tag
        } else if pattern.arguments.len() == 1 {
            pattern.arguments.get(0) // Regular tag
        } else {
            return Err(PreprocessorError::TooManyArguments(at));
        };
        self.push_non_breaking('<');
        self.push_str_non_breaking(&name);
        for Attribute(key, value) in pattern.iter_attributes() {
            match value.as_component() {
                None => {
                    self.push_non_breaking(' ');
                    self.push_str_non_breaking(key.as_str());
                }
                Some(value) => {
                    self.push_non_breaking(' ');
                    self.push_str_non_breaking(key.as_str());
                    self.push_str_non_breaking("=\"");
                    self.push_str_non_breaking(value.as_str());
                    self.push_non_breaking('"');
                }
                _ => return Err(PreprocessorError::IllegalAttributeValue(String::from(key), value.as_component().from())),
            };
        }
        self.push_non_breaking('>');
        if let Some(argument) = argument {
            self.write_inner(argument.as_composite())?;
        } else {
            return Ok(());
        };
        self.push_str_non_breaking("</");
        self.push_str_non_breaking(name);
        self.push_non_breaking('>');
        Ok(())
    }

    fn write_dictionary(&mut self, dictionary: &ParsedDictionary, at: Position) -> Result<(), PreprocessorError> {
        for Entry(key, value) in dictionary.iter() {
            self.push_non_breaking('<');
            self.push_str_non_breaking(key.as_str());
            self.push_non_breaking('>');
            self.write_inner(value)?;
            self.push_str_non_breaking("</");
            self.push_str_non_breaking(key.as_str());
            self.push_non_breaking('>');
        }
        Ok(())
    }

}

pub enum PreprocessorError {
    IllegalDictionary(Position),
    IllegalTable(Position),
    Custom(String),
    TooManyArguments(Position),
    IllegalAttributeValue(String, Position),
}
