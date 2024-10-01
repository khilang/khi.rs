use std::ops::Deref;
use crate::{Dictionary, Tagged, Value, Text, Element, Attribute, Compound, Tuple};
use crate::pdm::{ParsedDictionary, ParsedTaggedValue, ParsedTuple, ParsedValue, Position};

pub fn write_html(value: &ParsedValue) -> Result<String, PreprocessorError> {
    let mut output = String::new();
    let mut writer = XmlWriter { output: &mut output, column: 1, newline: 60, last: LastType::Whitespace, };
    writer.write_xml_compound(value)?;
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

    fn write_xml_compound(&mut self, value: &ParsedValue) -> Result<(), PreprocessorError> {
        match value {
            ParsedValue::Nil(..) => {}
            ParsedValue::Text(text, ..) => {
                self.push_str(text.as_str());
            }
            ParsedValue::Dictionary(dictionary, from, to) => {
                self.write_dictionary(dictionary, *from)?;
            }
            ParsedValue::List(table, from, to) => {
                return Err(PreprocessorError::IllegalTable(*from));
            }
            ParsedValue::Compound(compound, from, to) => {
                for element in compound.iter() {
                    if let Element::Element(value) = element {
                        self.write_xml_compound(value)?;
                    } else {
                        self.push_whitespace();
                    }
                }
            }
            ParsedValue::Tuple(tuple, from, to) => {
                if tuple.len() == 0 {
                    // Empty string
                } else {
                    return Err(PreprocessorError::IllegalTuple(*from));
                }
            }
            ParsedValue::Tagged(tag, from, to) => {
                self.write_tag(tag, *from)?;
            }
        }
        Ok(())
    }

    fn write_tag(&mut self, tag: &ParsedTaggedValue, at: Position) -> Result<(), PreprocessorError> {
        let name = tag.name();
        let inner_value = tag.get();
        if name.ends_with('!') {
            if name.deref() == "doctype!" {
                if tag.has_attributes() {
                    return Err(PreprocessorError::MacroError(format!("doctype! macro cannot have attributes.")))
                }
                if !inner_value.is_text() {
                    if inner_value.is_tuple() {eprintln!("{}", inner_value.as_tuple().unwrap().len())};
                    return Err(PreprocessorError::MacroError(format!("doctype! must have 1 text argument.")))
                }
                let doctype = inner_value;
                self.push_str_non_breaking("<!DOCTYPE ");
                self.push_str_non_breaking(doctype.as_text().unwrap().as_str());
                self.push_str_non_breaking(">");
                Ok(())
            } else if name.deref() == "raw!" {
                if let Some(text) = inner_value.as_text() {
                    self.output.push_str(text.as_str());
                    Ok(())
                } else {
                    Err(PreprocessorError::MacroError(format!("raw! can only take a text argument.")))
                }
            } else {
                Err(PreprocessorError::MacroError(format!("Unknown macro {}.", name)))
            }
        } else {
            self.push_non_breaking('<');
            self.push_str_non_breaking(&name);
            for Attribute(key, value) in tag.iter_attributes() {
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
            if inner_value.is_tuple() {//todo
                match inner_value.as_tuple().unwrap() {
                    ParsedTuple::Unit => return Ok(()), // Self closing tag
                    ParsedTuple::Single(s) => {
                        if s.is_unit() {
                            // Empty element
                        } else {
                            return Err(PreprocessorError::IllegalTuple(s.from()));
                        }
                    }
                    ParsedTuple::Multiple(..) => return Err(PreprocessorError::TooManyArguments(at)),
                }
            } else {
                self.write_xml_compound(inner_value)?
            }
            self.push_str_non_breaking("</");
            self.push_str_non_breaking(name);
            self.push_non_breaking('>');
            Ok(())
        }
    }

    fn write_dictionary(&mut self, dictionary: &ParsedDictionary, at: Position) -> Result<(), PreprocessorError> {
        for (key, value) in dictionary.iter() {
            self.push_non_breaking('<');
            self.push_str_non_breaking(key);
            self.push_non_breaking('>');
            self.write_xml_compound(value)?;
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
    IllegalTuple(Position),
}
