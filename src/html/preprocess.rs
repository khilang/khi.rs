use std::ops::Deref;
use crate::{Dictionary, Tag, Value, Text, Element, Entry, Attribute, Composition};
use crate::lex::Position;
use crate::parse::{ParsedDictionary, ParsedPattern, ParsedValue};

pub fn write_html(value: &ParsedValue) -> Result<String, PreprocessorError> {
    let mut output = String::new();
    let mut writer = XmlWriter { output: &mut output, column: 1, newline: 60, last: LastType::Whitespace, };
    writer.write_inner(value)?;
    Ok(output)
}

pub struct XmlWriter<'a> {
    output: &'a mut String,
    column: usize,
    newline: usize, // 0 for never newline.
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

}

impl XmlWriter<'_> {

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
            ParsedValue::Composition(composition, from, to) => {
                for element in composition.iter() {
                    if let Element::Solid(value) = element {
                        self.write_inner(value)?;
                    } else {
                        self.push_whitespace();
                    }
                }
            }
            ParsedValue::Pattern(pattern, from, to) => {
                self.write_tag(pattern, *from)?;
            }
        }
        Ok(())
    }

    fn write_tag(&mut self, pattern: &ParsedPattern, at: Position) -> Result<(), PreprocessorError> {
        let name = pattern.name();
        if name.ends_with('!') {
            if name.deref() == "doctype!" {
                if pattern.has_attributes() || pattern.len() != 1 {
                    return Err(PreprocessorError::MacroError(format!("doctype! macro cannot have attributes and must have 1 argument.")))
                }
                let doctype = pattern.get(0).unwrap();
                self.push_str_non_breaking("<!DOCTYPE ");
                self.push_str_non_breaking(doctype.as_text().unwrap().as_str());
                self.push_str_non_breaking(">");
                Ok(())
            } else if name.deref() == "raw!" {
                if let Some(raw) = pattern.get(0) {
                    if let Some(text) = raw.as_text() {
                        self.output.push_str(text.as_str());
                        Ok(())
                    } else {
                        Err(PreprocessorError::MacroError(format!("raw! can only take a text argument.")))
                    }
                } else {
                    Err(PreprocessorError::MacroError(format!("raw! must have one text argument.")))
                }
            } else {
                Err(PreprocessorError::MacroError(format!("Unknown macro {}.", name)))
            }
        } else {
            // Tag
            let argument = if pattern.len() == 0 {
                None // Self closing tag
            } else if pattern.len() == 1 {
                pattern.get(0) // Regular tag
            } else {
                return Err(PreprocessorError::TooManyArguments(at));
            };
            self.push_non_breaking('<');
            self.push_str_non_breaking(&name);
            for Attribute(key, value) in pattern.iter_attributes() {
                match value {
                    None => {
                        self.push_non_breaking(' ');
                        self.push_str_non_breaking(key);
                    }
                    Some(value) => {
                        self.push_non_breaking(' ');
                        self.push_str_non_breaking(key);
                        self.push_str_non_breaking("=\"");
                        self.push_str_non_breaking(value);
                        self.push_non_breaking('"');
                    }
                };
            }
            self.push_non_breaking('>');
            if let Some(argument) = argument {
                self.write_inner(argument)?;
            } else {
                return Ok(());
            };
            self.push_str_non_breaking("</");
            self.push_str_non_breaking(name);
            self.push_non_breaking('>');
            Ok(())
        }
    }

    fn write_dictionary(&mut self, dictionary: &ParsedDictionary, at: Position) -> Result<(), PreprocessorError> {
        for Entry(key, value) in dictionary.iter() {
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
    IllegalTable(Position),
    MacroError(String),
    TooManyArguments(Position),
}
