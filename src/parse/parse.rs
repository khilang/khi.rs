//! Khi parser reference implementation. Implements a tree-based parser.
//!
//! A document conforms to a value, dictionary or list. Use the corresponding
//! function to parse a document: [parse_value_str], [parse_dictionary_str] or
//! [parse_list_str].

// An O(n) predictive and recursive parser. Works in three stages: First, the
// input string is lexed and tokenized. Second, some tokens are grouped. Bracket
// groups are reduced to a single token, forming a token tree. Some tokens such
// as whitespace and tildes are removed and become attributes on the reduced
// tokens. In the final stage the parser operates on these reduced tokens to
// create a parsed document model (the AST).

use std::collections::HashSet;
use crate::parse::lex::{lex, LexError};
use crate::parse::parse::parser::{ParseError, Parser};
use crate::parse::parse::grouper::{ReduceError, Grouped, Reducer};
use crate::parse::pdm::{ParsedDictionary, ParsedList, ParsedValue, Position};

const MAX_DEPTH: usize = 256; //TODO

/// Parse a value document string.
pub fn parse_value_str(document: &str) -> Result<ParsedValue, Vec<ParseError>> {
    let tokens = unwrap_or_throw(tokenize(document))?;
    let mut strings = HashSet::new();
    let mut errors = Vec::new();
    let mut parser = Parser::new(&tokens, &mut strings, &mut errors, false, Position { index: 0, line: 0, column: 0 });
    let parse = parser.parse_value_document();
    if parse.is_ok() && !parser.is_end() {
        let error = ParseError::ExpectedEnd(parser.t0.to_type(), parser.at());
        errors.push(error);
    };
    present_parse(parse, errors)
}

/// Parse a dictionary document string.
pub fn parse_dictionary_str(document: &str) -> Result<ParsedDictionary, Vec<ParseError>> {
    let tokens = unwrap_or_throw(tokenize(document))?;
    let mut strings = HashSet::new();
    let mut errors = Vec::new();
    let mut parser = Parser::new(&tokens, &mut strings, &mut errors, false, Position { index: 0, line: 0, column: 0 });
    let parse = parser.parse_dictionary_document();
    if parse.is_ok() && !parser.is_end() {
        let error = ParseError::ExpectedEnd(parser.t0.to_type(), parser.at());
        errors.push(error);
    };
    present_parse(parse, errors)
}

/// Parse a list document string.
pub fn parse_list_str(document: &str) -> Result<ParsedList, Vec<ParseError>> {
    let tokens = unwrap_or_throw(tokenize(document))?;
    let mut strings = HashSet::new();
    let mut errors = Vec::new();
    let mut parser = Parser::new(&tokens, &mut strings, &mut errors, false, Position { index: 0, line: 0, column: 0 });
    let parse = parser.parse_list_document();
    if parse.is_ok() && !parser.is_end() {
        let error = ParseError::ExpectedEnd(parser.t0.to_type(), parser.at());
        errors.push(error);
    };
    present_parse(parse, errors)
}

fn unwrap_or_throw<T>(t: Result<T, ParseError>) -> Result<T, Vec<ParseError>> {
    match t {
        Ok(o) => Ok(o),
        Err(e) => Err(vec![e]),
    }
}

fn present_parse<T>(parse: Result<T, ParseError>, mut errors: Vec<ParseError>) -> Result<T, Vec<ParseError>> {
    match parse {
        Ok(d) => {
            if errors.is_empty() {
                Ok(d)
            } else {
                Err(errors)
            }
        }
        Err(e) => {
            errors.push(e);
            Err(errors)
        }
    }
}

/// Convert a Khi document to tokens.
fn tokenize(document: &str) -> Result<Vec<Grouped>, ParseError> {
    let chars = document.chars();
    let tokens = match lex(chars) {
        Ok(tokens) => tokens,
        Err(error) => {
            return match error {
                LexError::EscapeEos => Err(ParseError::EscapingEndOfStream),
                LexError::InvalidEscapeSequence(at) => Err(ParseError::InvalidEscapeSequence(at)),
                LexError::InvalidHashSequence(at) => Err(ParseError::IllegalHashSequence(at)),
                LexError::UnclosedTextBlock(at) => Err(ParseError::UnclosedTextBlock(at)),
                LexError::InvalidTextBlockConfiguration(at) => Err(ParseError::InvalidTextBlockConfiguration(at)),
                LexError::CannotCloseOpenTranscription(at) => Err(ParseError::CannotCloseOpenTranscription(at)),
            };
        }
    };
    let mut reducer = Reducer::new(&tokens);
    let reduced = match reducer.reduce() {
        Ok(tokens) => tokens,
        Err(error) => {
            return match error {
                ReduceError::MismatchedClose(close_type, close_at, in_type, in_at) => Err(ParseError::MismatchedClose(close_at, close_type.to_rule(), in_at, in_type.to_rule())),
            }
        }
    };
    Ok(reduced)
}

/// Parser
pub mod parser {
    use std::cmp::PartialEq;
    use std::collections::{HashMap, HashSet};
    use std::fmt::{Debug, Formatter};
    use std::ops::Deref;
    use std::rc::Rc;
    use std::slice::Iter;
    use std::vec;
    use crate::{Dictionary, Element, TextType, Value};
    use crate::parse::parse::grouper::{Grouped, ScopeType, StringType};
    use crate::parse::pdm::{ParsedAttribute, ParsedDictionary, ParsedList, ParsedTaggedTuple, ParsedText, ParsedValue, Position, ParsedTupleElement, ParsedTag, ParsedCatenation};

    pub struct Parser<'a> {
        stream: Iter<'a, Grouped>,
        pub t0: &'a Grouped,
        t1: &'a Grouped,
        strings: &'a mut HashSet<Rc<str>>,
        errors: &'a mut Vec<ParseError>,
        whitespace_before: bool,
        last_position: Position,
    }

    impl<'a> Parser<'a> {
        pub fn new(
            tokens: &'a Vec<Grouped>,
            strings: &'a mut HashSet<Rc<str>>,
            errors: &'a mut Vec<ParseError>,
            whitespace_before: bool,
            open_position: Position,
        ) -> Self {
            const DEFAULT: Grouped = Grouped::End(Position { index: 0, line: 0, column: 0 });
            let mut iter = Parser {
                stream: tokens.iter(),
                t0: &DEFAULT, t1: &DEFAULT,
                strings, errors, whitespace_before,
                last_position: open_position,
            };
            iter.shift();
            iter.shift();
            iter.whitespace_before = whitespace_before;
            iter.last_position = open_position;
            iter
        }

        fn shift(&mut self) {
            self.whitespace_before = self.t0.has_whitespace_after();
            self.last_position = self.t0.to();
            self.t0 = self.t1;
            self.t1 = self.stream.next().unwrap_or(self.t1);
        }

        pub fn at(&self) -> Position {
            self.t0.at()
        }

        pub fn at_last(&self) -> Position {
            self.last_position
        }

        pub(crate) fn is_end(&self) -> bool {
            matches!(self.t0, Grouped::End(..))
        }

        fn store_str(&mut self, string: &str) -> Rc<str> {
            if let Some(str) = self.strings.get(string) {
                str.clone()
            } else {
                let count = Rc::from(string);
                let str = Rc::clone(&count);
                self.strings.insert(count);
                str
            }
        }

    }

    impl Parser<'_> {

        /// Parse a value document.
        ///
        /// ```text
        /// <value-document> → *
        ///                  | *<value>*
        /// ```
        pub(crate) fn parse_value_document(&mut self) -> Result<ParsedValue, ParseError> {
            if !matches!(self.t0, Grouped::End(..)) {
                let (value, _is_tuple) = self.parse_value()?;
                Ok(value)
            } else {
                Ok(ParsedValue::TaggedTuple(ParsedTaggedTuple { tag: None, elements: vec![] }, self.t0.at(), self.t1.at()))
            }
        }

        /// Parse a dictionary document.
        ///
        /// ```text
        /// <dictionary-document> → *
        ///                       | *<dictionary>*
        /// ```
        pub(crate) fn parse_dictionary_document(&mut self) -> Result<ParsedDictionary, ParseError> {
            let dictionary = if matches!(self.t0, Grouped::AssignmentHeader(..) | Grouped::CurlyHeader(..) | Grouped::SquareHeader(..)) {
                self.parse_dictionary()?
            } else {
                ParsedDictionary::empty()
            };
            Ok(dictionary)
        }

        /// Parse a list document.
        ///
        /// ```text
        /// <list-document> → *
        ///                 | *<list>*
        /// ```
        pub(crate) fn parse_list_document(&mut self) -> Result<ParsedList, ParseError> {
            let list = if matches!(self.t0, Grouped::String(..) | Grouped::CurlyBracket(..) | Grouped::SquareBracket(..) | Grouped::AngleBracket(..) | Grouped::Tilde(..) | Grouped::Bar(..) | Grouped::RightAngle(..) | Grouped::TaggedValueHeader(..)) {
                self.parse_list()?
            } else {
                ParsedList::empty()
            };
            Ok(list)
        }

        /// Parse a value.
        ///
        /// Returns a true bool if a tuple matches.
        ///
        /// ```text
        /// <value> → <catenation>
        ///         | <barred-value>
        ///         | <tagged-value>
        ///
        /// <barred-value>  → <catenation> <barred-value'>
        ///                 | <barred-value'>
        /// <barred-value'> → "|" <catenation>
        ///                 | "|" <catenation> <barred-value'>
        /// ```
        fn parse_value(&mut self) -> Result<(ParsedValue, bool), ParseError> {
            let from = self.at();
            if matches!(self.t0, Grouped::TaggedValueHeader(..)) {
                return Ok((self.parse_tagged_value()?, false));
            }
            let mut elements = vec![];
            match self.t0 {
                Grouped::String(..) | Grouped::CurlyBracket(..) | Grouped::SquareBracket(..) | Grouped::AngleBracket(..) | Grouped::Tilde(..) => {
                    let element = self.parse_catenation()?;
                    if !matches!(self.t0, Grouped::Bar(..)) || !matches!(self.t1, Grouped::String(..) | Grouped::CurlyBracket(..) | Grouped::SquareBracket(..) | Grouped::AngleBracket(..) | Grouped::Tilde(..)) {
                        return Ok((element, false));
                    }
                    elements.push(ParsedTupleElement::Positional(element));
                    self.shift();
                },
                Grouped::Bar(..) => {
                    self.shift();
                }
                _ => return ParseError::token_expectation_error(&[Rule::Catenation, Rule::Tuple, Rule::TaggedValue], self.t0, Rule::Value, self.t0.at()),
            }
            loop {
                let element = ParsedTupleElement::Positional(self.parse_catenation()?);
                elements.push(element);
                if !matches!(self.t0, Grouped::Bar(..)) || !matches!(self.t1, Grouped::String(..) | Grouped::CurlyBracket(..) | Grouped::SquareBracket(..) | Grouped::AngleBracket(..) | Grouped::Tilde(..)) {
                    break;
                }
                self.shift();
            }
            let to = self.at();
            Ok((ParsedValue::TaggedTuple(ParsedTaggedTuple { tag: None, elements }, from, to), true))
        }

        /// Parse text.
        ///
        /// ```text
        /// <text> → <words>
        ///        | [closed-transcription]
        ///        | [unclosed-transcription]
        ///        | [text-block]
        ///
        /// <words>  → [word]
        ///          | [word] <words>
        /// ```
        fn parse_text(&mut self) -> Result<ParsedValue, ParseError> {
            let from = self.at();
            if let Grouped::String(from, to, b, string_type, content) = self.t0 {
                match string_type {
                    StringType::ClosedTranscription | StringType::UnclosedTranscription | StringType::TextBlock => {
                        self.shift();
                        let to = self.at();
                        let str = self.store_str(&content);
                        let text = ParsedText { str, text_type: TextType::Literal };
                        Ok(ParsedValue::Text(text, *from, to))
                    }
                    StringType::Word => {
                        let mut text = String::new();
                        let mut space_before = false;
                        loop {
                            match self.t0 {
                                Grouped::String(.., b, strtype, string) => {
                                    if *strtype != StringType::Word {
                                        break;
                                    }
                                    if space_before {
                                        text.push(' ');
                                    }
                                    text.push_str(string);
                                    space_before = *b;
                                    self.shift();
                                }
                                _ => break,
                            }
                        }
                        let to = self.at();
                        let str = self.store_str(&text);
                        let text = ParsedText { str, text_type: TextType::Plain };
                        Ok(ParsedValue::Text(text, *from, to))
                    }
                }
            } else {
                ParseError::token_expectation_error(&[Rule::String], self.t0, Rule::Text, self.t0.at())
            }
        }

        /// Parse confined text.
        ///
        /// ```text
        /// <confined-text> → [word]
        ///                 | [closed-transcription]
        ///                 | [open-transcription]
        ///                 | [text-block]
        /// ```
        fn parse_confined_text(&mut self) -> Result<Rc<str>, ParseError> {
            match self.t0 {
                Grouped::String(.., text) => {
                    self.shift();
                    Ok(self.store_str(text))
                }
                _ => return ParseError::token_expectation_error(&[Rule::Word, Rule::Transcription, Rule::TextBlock], self.t0, Rule::String, self.at()),
            }
        }

        /// Parse a tagged value.
        ///
        /// ```text
        /// <tagged-value> → <tag>":"_<value>
        /// ```
        fn parse_tagged_value(&mut self) -> Result<ParsedValue, ParseError> {
            let from = self.at();
            let tag = self.parse_tag()?;
            if !matches!(self.t0, Grouped::Colon(..)) {
                return ParseError::token_expectation_error(&[Rule::Colon], self.t0, Rule::TaggedValue, from);
            };
            self.require_whitespace_after();
            self.shift();
            let (value, is_tuple) = self.parse_value()?;
            let to = self.at();
            let tuple = if !is_tuple { // Wrap in tuple if not tuple phrase.
                ParsedTaggedTuple { tag: None, elements: vec![ParsedTupleElement::Positional(value)] }
            } else {
                match value {
                    ParsedValue::TaggedTuple(t, _, _) => {
                        t
                    }
                    _ => unreachable!(),
                }
            };
            let tagged_value = match tag {
                Some((name, attributes)) => {
                    ParsedValue::TaggedTuple(ParsedTaggedTuple {
                        tag: Some(ParsedTag { name: self.store_str(&name), attributes }),
                        elements: tuple.elements,
                    }, from, to)
                }
                None => ParsedValue::TaggedTuple(tuple, from, to),
            };
            Ok(tagged_value)
        }

        /// Parse a dictionary.
        ///
        /// ```text
        /// <dictionary> → <delimited-dictionary>
        ///              | <aligned-dictionary>
        ///              | <sectioned-dictionary>
        ///
        /// <sectioned-dictionary>  → <sectioned-dictionary'>
        ///                         | <regular-dictionary>_<sectioned-dictionary'>
        /// <sectioned-dictionary'> → <section>
        ///                         | <section>_<sectioned-dictionary'>
        ///
        /// <section> → <curly-header>_<value>
        ///           | <curly-header>
        ///           | <curly-header>_<regular-dictionary>
        ///           | <square-header>
        ///           | <square-header>_<list>
        /// ```
        fn parse_dictionary(&mut self) -> Result<ParsedDictionary, ParseError> {
            let mut dictionary_sections = vec![];
            let top_level_assign;
            let from = self.at();
            if matches!(self.t0, Grouped::AssignmentHeader(..)) {
                let mut entries = self.parse_nonsectioned_dictionary()?;
                let dict = dict_from_entries(entries, from)?;
                top_level_assign = dict;
                if matches!(self.t0, Grouped::CurlyHeader(..) | Grouped::SquareHeader(..)) {
                    self.require_whitespace_before();
                }
            } else {
                top_level_assign = HashMap::new();
            }
            if matches!(self.t0, Grouped::CurlyHeader(..) | Grouped::SquareHeader(..)) {
                loop {
                    let (header, section, append) = match self.t0 {
                        Grouped::CurlyHeader(..) => {
                            let (header, append) = self.parse_header()?;
                            let content_from = self.at();
                            let section = match self.t0 {
                                Grouped::String(..) | Grouped::CurlyBracket(..) | Grouped::SquareBracket(..) | Grouped::AngleBracket(..) | Grouped::Tilde(..) | Grouped::Bar(..) | Grouped::TaggedValueHeader(..) => {
                                    self.require_whitespace_before();
                                    let (value, _is_tuple) = self.parse_value()?;
                                    if let Some(d) = value.as_dictionary() {
                                        Section::Dictionary { subsections: HashMap::new(), assign: d.clone() }
                                    } else {
                                        Section::Value { value }
                                    }
                                }
                                Grouped::AssignmentHeader(..) => {
                                    self.require_whitespace_before();
                                    let entries = self.parse_nonsectioned_dictionary()?;
                                    let dict = dict_from_entries(entries, content_from)?;
                                    Section::Dictionary { subsections: HashMap::new(), assign: ParsedDictionary { entries: dict } }
                                }
                                _ => Section::Dictionary { subsections: HashMap::new(), assign: ParsedDictionary { entries: HashMap::new() } },
                            };
                            (header, section, append)
                        }
                        Grouped::SquareHeader(..) => {
                            let (header, append) = self.parse_header()?;
                            let content_from = self.at();
                            let section = if matches!(self.t0, Grouped::String(..) | Grouped::CurlyBracket(..) | Grouped::SquareBracket(..) | Grouped::AngleBracket(..) | Grouped::Tilde(..) | Grouped::Bar(..) | Grouped::TaggedValueHeader(..)) {
                                self.require_whitespace_before();
                                let list = self.parse_list()?;
                                let table_to = self.at_last();
                                Section::List { values: list.elements }
                            } else {
                                Section::List { values: vec![] }
                            };
                            (header, section, append)
                        }
                        _ => break,
                    };
                    dictionary_sections.push((header, section, append));
                    if matches!(self.t0, Grouped::CurlyHeader(..) | Grouped::SquareHeader(..)) {
                        self.require_whitespace_before();
                    }
                }
            }
            let to = self.at_last();
            let sections = make_sections(dictionary_sections, from)?;
            let dictionary = join_sections(sections, top_level_assign, from)?;
            Ok(dictionary)
        }

        /// Parse a regular dictionary.
        ///
        /// ```text
        /// <regular-dictionary> → <delimited-dictionary>
        ///                      | <aligned-dictionary>
        /// ```
        fn parse_nonsectioned_dictionary(&mut self) -> Result<Vec<ParsedEntry>, ParseError> {
            let entry = self.parse_entry()?;
            let entries = match self.t0 {
                Grouped::Semicolon(..) => {
                    self.shift();
                    if matches!(self.t0, Grouped::AssignmentHeader(..)) {
                        self.parse_delimited_dictionary(vec![entry])?
                    } else {
                        vec![entry]
                    }
                }
                Grouped::AssignmentHeader(..) => {
                    self.require_whitespace_before();
                    self.parse_aligned_dictionary(vec![entry])?
                }
                _ => vec![entry],
            };
            Ok(entries)
        }

        /// Parse a delimited dictionary.
        ///
        /// ```text
        /// <delimited-dictionary> → <entry>
        ///                        | <entry> ";"
        ///                        | <entry> ";" <delimited-dictionary>
        /// ```
        fn parse_delimited_dictionary(&mut self, entries: Vec<ParsedEntry>) -> Result<Vec<ParsedEntry>, ParseError> {
            let mut entries = entries;
            loop {
                let entry = self.parse_entry()?;
                entries.push(entry);
                if !matches!(self.t0, Grouped::Semicolon(..)) {
                    break;
                }
                self.shift();
                if !matches!(self.t0, Grouped::AssignmentHeader(..)) {
                    break;
                }
            }
            Ok(entries)
        }

        /// Parse an aligned dictionary.
        ///
        /// ```text
        /// <aligned-dictionary> → <entry>
        ///                      | <entry>_<aligned-dictionary>
        /// ```
        fn parse_aligned_dictionary(&mut self, entries: Vec<ParsedEntry>) -> Result<Vec<ParsedEntry>, ParseError> {
            let mut entries = entries;
            loop {
                let entry = self.parse_entry()?;
                entries.push(entry);
                if !matches!(self.t0, Grouped::AssignmentHeader(..)) {
                    break;
                }
                self.require_whitespace_before();
            }
            Ok(entries)
        }

        /// Parse a dictionary header.
        ///
        /// ```text
        /// <curly-header> → "{"<header-key>"}"":"
        ///
        /// <square-header> → "["<header-key>"]"":"
        /// ```
        fn parse_header(&mut self) -> Result<(ParsedHeaderKey, bool), ParseError> {
            let mut segmented_key = vec![];
            let mut append = false;
            let at = self.at();
            match self.t0 {
                Grouped::CurlyHeader(_, ht, fw, scope) | Grouped::SquareHeader(_, ht, fw, scope) => {
                    let mut parser = Parser::new(&scope, self.strings, self.errors, *fw, *ht);
                    parser.require_no_whitespace_before();
                    (segmented_key, append) = parser.parse_header_key()?;
                    parser.require_no_whitespace_before();
                    parser.require_end();
                }
                _ => return ParseError::token_expectation_error(&[Rule::Key], self.t0, Rule::Header, at),
            }
            self.shift();
            if !matches!(self.t0, Grouped::Colon(..)) {
                return ParseError::token_expectation_error(&[Rule::Colon], self.t0, Rule::AbsoluteDictionary, self.at());
            }
            self.require_no_whitespace_before();
            self.shift();
            Ok((segmented_key, append))
        }

        /// Parse a header key.
        ///
        /// ```text
        /// <header-key> → <key>
        ///              | <key>"[""]"
        ///              | <key>_">"_<header-key>
        /// ```
        fn parse_header_key(&mut self) -> Result<(Vec<Rc<str>>, bool), ParseError> {
            let mut key = vec![];
            let mut append = false;
            loop {
                match self.t0 {
                    Grouped::String(.., s) | Grouped::AssignmentHeader(.., s) => {
                        let k = self.store_str(s);
                        key.push(k);
                    }
                    _ => return ParseError::token_expectation_error(&[Rule::String], self.t0, Rule::Key, self.t0.at())
                }
                self.shift();
                if let Grouped::SquareBracket(a,b,c,d,e) = self.t0 {
                    self.require_no_whitespace_before();
                    if !e.len() == 1 || *c { // Empty: only End token and no whitespace
                        return ParseError::token_expectation_error(&[Rule::Square], self.t0, Rule::Header, self.t0.at())
                    }
                    self.shift();
                    append = true;
                    break;
                }
                if !matches!(self.t0, Grouped::RightAngle(..)) {
                    break;
                }
                self.require_whitespace_before();
                self.require_whitespace_after();
                self.shift();
            }
            Ok((key, append))
        }

        /// Parse an entry.
        ///
        /// ```text
        /// <entry> → <key>":"_<value>
        /// ```
        fn parse_entry(&mut self) -> Result<ParsedEntry, ParseError> {
            let at = self.at();
            if let Grouped::AssignmentHeader(.., s) | Grouped::String(.., s) = self.t0 {
                let key = self.parse_entry_key()?;
                self.require_whitespace_before();
                let (value, _is_tuple) = self.parse_value()?;
                Ok((key, value))
            } else {
                return ParseError::token_expectation_error(&[Rule::Key], self.t0, Rule::Entry, at);
            }
        }

        /// Parse a key and a colon.
        fn parse_entry_key(&mut self) -> Result<ParsedKey, ParseError> { // TODO TODO TODO TODO TODO
            //let mut key = vec![];
            //loop {
            //
            //    let k = self.store_str(s);
            //    key.push(k);
            //    self.shift();
            //    if !matches!(self.t0, Grouped::Colon(..)) {
            //        return ParseError::token_expectation_error(&[Rule::Colon], self.t0, Rule::Key, self.t0.at());
            //    }
            //    self.require_no_whitespace_before();
            //    self.shift();
            //    if !matches!(self.t0, Grouped::String(..) | Grouped::AssignmentHeader(..)) || !matches!(self.t1, Grouped::Colon(..)) {
            //        break;
            //    }
            //    self.require_no_whitespace_before();
            //}
            let s = match self.t0 {
                Grouped::String(_, _, _, _, s) => s,
                Grouped::AssignmentHeader(_, _, _, s) => s,
                _ => return ParseError::token_expectation_error(&[Rule::String], self.t0, Rule::Key, self.t0.at()),
            };
            let k = self.store_str(s);
            self.shift();
            if !matches!(self.t0, Grouped::Colon(..)) {
                return ParseError::token_expectation_error(&[Rule::Colon], self.t0, Rule::Key, self.t0.at());
            }
            self.require_no_whitespace_before();
            self.shift();
            Ok(k)
        }

        /// Parse a list.
        ///
        /// ```text
        /// <list> → <delimited-list>
        ///        | <bulleted-list>
        ///        | <tagged-list>
        ///        | <tabulated-list>
        /// ```
        fn parse_list(&mut self) -> Result<ParsedList, ParseError> {
            match self.t0 {
                Grouped::RightAngle(..) => self.parse_bulleted_list(vec![]),
                Grouped::Bar(..) => {
                    let (value, _) = self.parse_value()?;
                    if matches!(self.t0, Grouped::Bar(..)) {
                        self.shift();
                        if matches!(self.t0, Grouped::Bar(..)) {
                            self.require_whitespace_before();
                            self.parse_tabular_list(vec![value])
                        } else { // One row tabulated list.
                            Ok(ParsedList { elements: vec![value] })
                        }
                    } else if matches!(self.t0, Grouped::Semicolon(..)) {
                        self.shift();
                        self.parse_delimited_list(vec![value])
                        // TODO: Set from
                    } else {
                        Ok(ParsedList { elements: vec![value] })
                    }
                }
                Grouped::String(..) | Grouped::Tilde(..) | Grouped::CurlyBracket(..) | Grouped::SquareBracket(..) | Grouped::AngleBracket(..) => {
                    self.parse_delimited_list(vec![])
                }
                Grouped::TaggedValueHeader(..) => {
                    let value = self.parse_tagged_value()?;
                    if matches!(self.t0, Grouped::TaggedValueHeader(..)) {
                        self.require_whitespace_before();
                        self.parse_tagged_list(vec![value])
                    } else if matches!(self.t0, Grouped::Semicolon(..)) {
                        self.shift();
                        self.parse_delimited_list(vec![value])
                    } else {
                        Ok(ParsedList { elements: vec![value] })
                    }
                }
                _ => return ParseError::token_expectation_error(&[Rule::DelimitedList, Rule::AlignedList, Rule::TabularList, Rule::TaggedList], self.t0, Rule::List, self.t0.at()),
            }
        }

        /// Parse a delimited list.
        ///
        /// ```text
        /// <delimited-list> → <value>
        ///                  | <value> ";"
        ///                  | <value> ";" <delimited-list>
        /// ```
        fn parse_delimited_list(&mut self, elements: Vec<ParsedValue>) -> Result<ParsedList, ParseError> {
            let mut elements = elements;
            loop {
                let (value, _is_tuple) = self.parse_value()?;
                elements.push(value);
                if !matches!(self.t0, Grouped::Semicolon(..)) {
                    break;
                }
                self.shift();
                if !matches!(self.t0, Grouped::String(..) | Grouped::Tilde(..) | Grouped::Bar(..) | Grouped::CurlyBracket(..) | Grouped::SquareBracket(..) | Grouped::AngleBracket(..) | Grouped::TaggedValueHeader(..)) {
                    break;
                }
            }
            let list = ParsedList { elements };
            Ok(list)
        }

        /// Parse a bulleted list.
        ///
        /// ```text
        /// <bulleted-list> → ">"_<value>
        ///                 | ">"_<value>_<bulleted-list>
        ///                 | <tagged-tuple>
        ///                 | <tagged-tuple>_<bulleted-list>
        /// ```
        fn parse_bulleted_list(&mut self, elements: Vec<ParsedValue>) -> Result<ParsedList, ParseError> {
            let mut elements = elements;
            let at = self.at(); // TODO: Might be earlier.
            if !matches!(self.t0, Grouped::RightAngle(..)) {
                return ParseError::token_expectation_error(&[Rule::RightAngle], self.t0, Rule::AlignedList, at);
            }
            loop {
                match self.t0 {
                    Grouped::RightAngle(_, _) => {
                        self.require_whitespace_after();
                        self.shift();
                        if matches!(self.t0, Grouped::Bar(..)) {
                            unimplemented!(); // TODO: Bar is not allowed here.
                        }
                        let (value, is_tuple) = self.parse_value()?;
                        let to = value.to();
                        elements.push(value);
                    }
                    _ => unreachable!(),
                }
                if !matches!(self.t0, Grouped::RightAngle(..)) {
                    break;
                }
                self.require_whitespace_before();
            }
            Ok(ParsedList { elements })
        }

        /// Parse a tagged list.
        ///
        /// ```text
        /// <tagged-list> → <tagged-tuple>
        ///               | <tagged-tuple>_<bulleted-list>
        /// ```
        fn parse_tagged_list(&mut self, elements: Vec<ParsedValue>) -> Result<ParsedList, ParseError> {
            let mut elements = elements;
            let at = self.at(); // TODO: Might be earlier.
            if !matches!(self.t0, Grouped::TaggedValueHeader(..)) {
                return ParseError::token_expectation_error(&[Rule::TaggedValue], self.t0, Rule::AlignedList, at);
            }
            loop {
                match self.t0 {
                    Grouped::TaggedValueHeader(_, _, _, _) => {
                        let value = self.parse_tagged_value()?;
                        elements.push(value);
                    }
                    _ => unreachable!(),
                }
                if !matches!(self.t0, Grouped::TaggedValueHeader(..)) {
                    break;
                }
                self.require_whitespace_before();
            }
            Ok(ParsedList { elements })
        }

        /// Parse a tabulated list.
        ///
        /// ```text
        /// <tabulated-list> → <tuple'> "|"
        ///                  | <tuple'> "|"_<tabulated-list>
        /// ```
        fn parse_tabular_list(&mut self, elements: Vec<ParsedValue>) -> Result<ParsedList, ParseError> {
            let mut elements = elements;
            if !matches!(self.t0, Grouped::Bar(..)) {
                return ParseError::token_expectation_error(&[Rule::Bar], self.t0, Rule::TabularList, self.t0.at());
            }
            loop {
                let (value, _) = self.parse_value()?;
                elements.push(value);
                if !matches!(self.t0, Grouped::Bar(..)) {
                    return ParseError::token_expectation_error(&[Rule::Bar], self.t0, Rule::TabularList, self.t0.at());
                }
                self.shift();
                if !matches!(self.t0, Grouped::Bar(..)) {
                    break;
                }
                self.require_whitespace_before();
            }
            Ok(ParsedList { elements })
        }

        /// Parse tagged arguments.
        ///
        /// ```text
        /// <tagged-arguments> → <tag>
        ///                    | <tag><arguments>
        /// ```
        fn parse_confined_tuple(&mut self) -> Result<ParsedValue, ParseError> {
            let from = self.at();
            let tag = self.parse_tag()?;
            let arguments = if matches!(self.t0, Grouped::Colon(..)) {
                self.parse_confined_values()?
            } else {
                vec![]
            };
            let to = self.at();
            let tagged_arguments = match tag {
                Some((name, attributes)) => {
                    let name = self.store_str(&name);
                    ParsedValue::TaggedTuple(ParsedTaggedTuple {
                        tag: Some(ParsedTag { name, attributes }),
                        elements: arguments
                    }, from, to)
                }
                None => ParsedValue::TaggedTuple(ParsedTaggedTuple { tag: None, elements: arguments }, from, to),
            };
            Ok(tagged_arguments)
        }

        /// Parse arguments.
        ///
        /// ```text
        /// <confined-values> → ":"<confined-catenation>
        ///                   | ":"<confined-catenation><confined-values>
        /// ```
        /// `<confined-value>` takes precedence over `<confined-values>` on ambiguity.
        ///
        /// ```text
        /// <confined-catenation> → <confined-value>
        ///                       | [word]<confined-value>
        /// ```
        ///
        /// ```text
        /// <confined-value> → <confined-text>
        ///                  | <value-bracket>
        ///                  | <dictionary-bracket>
        ///                  | <list-bracket>
        ///                  | <confined-tuple>
        /// ```
        fn parse_confined_values(&mut self) -> Result<Vec<ParsedTupleElement>, ParseError> {
            let mut arguments = vec![];
            if !matches!(self.t0, Grouped::Colon(..)) {
                return ParseError::token_expectation_error(&[Rule::Colon], self.t0, Rule::Arguments, self.t0.at());
            }
            loop {
                let from = self.at();
                self.require_no_whitespace_after();
                self.shift();
                match self.t0 {
                    Grouped::String(from, to, .., typ, s) => {
                        self.shift();
                        let str = self.store_str(s);
                        let text_type = match *typ {
                            StringType::Word => TextType::Plain,
                            StringType::ClosedTranscription => TextType::Literal,
                            StringType::UnclosedTranscription => TextType::Literal,
                            StringType::TextBlock => TextType::Literal,
                        };
                        let text = ParsedValue::Text(ParsedText { str, text_type }, *from, *to);
                        if !self.whitespace_before && *typ == StringType::Word {
                            let arg2 = match self.t0 {
                                Grouped::CurlyBracket(..) => {
                                    Some(self.parse_curly_bracket()?)
                                }
                                Grouped::SquareBracket(..) => {
                                    Some(self.parse_list_bracket()?)
                                }
                                Grouped::AngleBracket(..) => {
                                    Some(self.parse_confined_tuple()?)
                                    //break;
                                }
                                _ => None,
                            };
                            if let Some(arg2) = arg2 {
                                let to = self.at();
                                arguments.push(ParsedTupleElement::Positional(ParsedValue::Catenation(ParsedCatenation { components: vec![Element::Element(text, true), Element::Element(arg2, false)] }, *from, to)));
                            } else {
                                arguments.push(ParsedTupleElement::Positional(text));
                            }
                        } else {
                            arguments.push(ParsedTupleElement::Positional(text));
                        }
                    }
                    Grouped::CurlyBracket(..) => {
                        let value = self.parse_curly_bracket()?;
                        arguments.push(ParsedTupleElement::Positional(value));
                    }
                    Grouped::SquareBracket(..) => {
                        let list = self.parse_list_bracket()?;
                        arguments.push(ParsedTupleElement::Positional(list));
                    }
                    Grouped::AngleBracket(..) => {
                        let argument = self.parse_confined_tuple()?;
                        arguments.push(ParsedTupleElement::Positional(argument));
                        //break;
                    }
                    _ => return ParseError::token_expectation_error(&[Rule::String, Rule::BracketedValue, Rule::BracketedDictionary, Rule::BracketedList, Rule::TaggedArguments], self.t0, Rule::Argument, self.t0.at()),
                }
                if !matches!(self.t0, Grouped::Colon(..)) {
                    break;
                }
                self.require_no_whitespace_before();
            }
            Ok(arguments)
        }

        /// Parse a tag.
        ///
        /// ```text
        /// <tag> → "<"<key>">"
        ///       | "<"<key>_<attributes> ">"
        ///       | "<"">"
        /// ```
        fn parse_tag(&mut self) -> Result<Option<(Rc<str>, Vec<ParsedAttribute>)>, ParseError> {
            if let Grouped::AngleBracket(from, _, fw, _, scope) | Grouped::TaggedValueHeader(from, _, fw, scope) = self.t0 {
                self.shift();
                let mut parser = Parser::new(scope, self.strings, self.errors, *fw, *from);
                parser.require_no_whitespace_before();
                if parser.is_end() {
                    return Ok(None);
                }
                let name = match parser.t0 {
                    Grouped::String(from, _, _, t, name) | Grouped::AssignmentHeader(from, _, t, name) => {
                        if *t != StringType::Word && *t != StringType::ClosedTranscription {
                            parser.errors.push(ParseError::TagNameMustBeKey(*from, parser.t0.to_type()));
                        }
                        parser.store_str(name)
                    }
                    _ => return ParseError::token_expectation_error(&[Rule::Name], &parser.t0, Rule::Tag, *from),
                };
                parser.shift();
                let attributes = if matches!(parser.t0, Grouped::String(..) | Grouped::AssignmentHeader(..)) {
                    parser.parse_attributes()?
                } else {
                    parser.require_no_whitespace_before();
                    vec![]
                };
                parser.require_end();
                Ok(Some((name, attributes)))
            } else {
                return ParseError::token_expectation_error(&[Rule::AngularBracket], self.t0, Rule::Tag, self.at());
            }
        }

        /// Parse attributes.
        ///
        /// ```text
        /// <attributes> → <attribute>
        ///              | <attribute>_<attributes>
        /// ```
        ///
        /// ```text
        /// <attribute> → <key>
        ///             | <key>"="<string>
        /// ```
        fn parse_attributes(&mut self) -> Result<Vec<ParsedAttribute>, ParseError> {
            let mut attributes = vec![];
            if !matches!(self.t0, Grouped::String(..)) {
                return ParseError::token_expectation_error(&[Rule::Attribute], self.t0, Rule::Attributes, self.at());
            }
            loop {
                let at = self.at();
                let split = match self.t0 {
                    Grouped::String(_, _, _, t, key) => {
                        if *t == StringType::Word {
                            let split = key.split("=").collect::<Vec<&str>>();
                            self.shift();
                            split
                        } else if *t == StringType::ClosedTranscription {
                            let mut split = vec![];
                            split.push(key);

                            let split = key.split("=").collect::<Vec<&str>>();
                            self.shift();
                            split
                        } else {
                            return Err(ParseError::AttributeMustBeKey(self.at(), self.t0.to_type()));
                        }
                    }
                    _ => break,
                };
                if split.len() == 1 {
                    let key = split[0];
                    attributes.push(ParsedAttribute::Flag(self.store_str(key)));
                } else if split.len() == 2 {
                    let key = split[0];
                    let value = split[1];
                    if key.is_empty() {
                        return Err(ParseError::AttributeIllegalEquals(at));
                    }
                    let key = self.store_str(key);
                    if value.is_empty() {
                        let value = self.parse_confined_text()?;
                        attributes.push(ParsedAttribute::Valued(key, value));
                    } else {
                        let value = self.store_str(value);
                        attributes.push(ParsedAttribute::Valued(key, value));
                    }
                } else {
                    return Err(ParseError::AttributeIllegalEquals(at));
                }
            }
            Ok(attributes)
        }

        /// Parse a catenation.
        ///
        /// ```text
        /// <catenation> → <term>
        ///              | <term> <catenation>
        ///              | "~"
        /// ```
        /// `<term>` takes precedence over `<catenation>` on ambiguity, which can occur
        /// with `~` when `<term>` matches `<text>`.
        ///
        /// ```text
        /// <term> → <text>
        ///        | <bracketed-value>
        ///        | <bracketed-dictionary>
        ///        | <bracketed-list>
        ///        | <tagged-arguments>
        /// ```
        fn parse_catenation(&mut self) -> Result<ParsedValue, ParseError> {
            let mut terms: Vec<ParsedValue> = vec![];
            let mut whitespace = vec![];
            let mut bracketed = vec![];
            let from = self.at();
            let mut space_before = false;
            if !matches!(self.t0, Grouped::String(..) | Grouped::CurlyBracket(..) | Grouped::SquareBracket(..) | Grouped::AngleBracket(..) | Grouped::Tilde(..)) {
                return ParseError::token_expectation_error(&[Rule::String, Rule::BracketedDictionary, Rule::BracketedValue, Rule::BracketedList, Rule::TaggedArguments, Rule::Tilde], self.t0, Rule::Catenation, self.t0.at());
            }
            if matches!(self.t0, Grouped::Tilde(..)) {
                self.shift();
            } else {
                loop {
                    space_before = self.whitespace_before;
                    match self.t0 {
                        Grouped::String(f, fg, h, hj, typ) => {
                            let bracket = *hj != StringType::Word;
                            let text = self.parse_text()?;
                            push_term(&mut terms, &mut bracketed, &mut whitespace, text, space_before, bracket);
                        }
                        Grouped::CurlyBracket(..) => {
                            let mut value = self.parse_curly_bracket()?;
                            if value.is_text() { // Track if open text has brackets around it.
                                let text = value.as_mut_text().unwrap();
                                if text.text_type == TextType::Plain {
                                    text.text_type = TextType::Bracket;
                                }
                            }
                            push_term(&mut terms, &mut bracketed, &mut whitespace, value, space_before, true);
                        }
                        Grouped::SquareBracket(..) => {
                            let value = self.parse_list_bracket()?;
                            push_term(&mut terms, &mut bracketed, &mut whitespace, value, space_before, true);
                        },
                        Grouped::AngleBracket(..) => {
                            let value = self.parse_confined_tuple()?;
                            push_term(&mut terms, &mut bracketed, &mut whitespace, value, space_before, true);
                        },
                        _ => break,
                    }
                }
            }
            let to = self.at();
            return Ok(ParsedValue::from_terms(from, to, terms, whitespace, bracketed));
            fn push_term(terms: &mut Vec<ParsedValue>, bracketed: &mut Vec<bool>, whitespace: &mut Vec<bool>, component: ParsedValue, ws_before: bool, bracket: bool) {
                bracketed.push(bracket);
                if terms.len() != 0 {
                    if ws_before {
                        whitespace.push(true);
                    } else {
                        whitespace.push(false);
                    }
                };
                terms.push(component);
            }
        }

        fn parse_key(&mut self) -> Result<String, ParseError> {
            unimplemented!() // TODO
        }

        /// Parse a bracketed value or dictionary.
        ///
        /// ```text
        /// <bracketed-value> → "{" <value> "}"
        ///
        /// <bracketed-dictionary> → "{" "}"
        ///                        | "{" <dictionary> "}"
        /// ```
        fn parse_curly_bracket(&mut self) -> Result<ParsedValue, ParseError> {
            if let Grouped::CurlyBracket(from, to, wi, _, scope) = self.t0 {
                self.shift();
                let mut parser = Parser::new(scope, self.strings, self.errors, *wi, *to);
                let value = match parser.t0 {
                    Grouped::AssignmentHeader(..) | Grouped::CurlyHeader(..) | Grouped::SquareHeader(..) => {
                        let dictionary = parser.parse_dictionary()?;
                        ParsedValue::Dictionary(dictionary, *from, *to)
                    }
                    Grouped::String(..) | Grouped::CurlyBracket(..) | Grouped::SquareBracket(..) | Grouped::AngleBracket(..) | Grouped::Bar(..) | Grouped::Tilde(..) | Grouped::TaggedValueHeader(..) => {
                        let (value, _is_tuple) = parser.parse_value()?;
                        value
                    }
                    Grouped::End(..) => {
                        let dictionary = ParsedDictionary::empty();
                        ParsedValue::Dictionary(dictionary, *from, *to)
                    }
                    _ => return ParseError::token_expectation_error(&[Rule::Value, Rule::Dictionary], parser.t0, Rule::Bracket, *from),
                };
                parser.require_end();
                Ok(value)
            } else {
                return ParseError::token_expectation_error(&[Rule::BracketOpen], self.t0, Rule::Bracket, self.at());
            }
        }

        /// Parse bracketed list.
        ///
        /// ```text
        /// <bracketed-list> → "[" "]"
        ///                  | "[" <list> "]"
        /// ```
        fn parse_list_bracket(&mut self) -> Result<ParsedValue, ParseError> {
            if let Grouped::SquareBracket(from, to, fw, _, scope) = &self.t0 {
                self.shift();
                let mut parser = Parser::new(scope, self.strings, self.errors, *fw, *to);
                let list = if !parser.is_end() {
                    parser.parse_list()?
                } else {
                    ParsedList::empty()
                };
                parser.require_end();
                Ok(ParsedValue::List(list, *from, *to))
            } else {
                return ParseError::token_expectation_error(&[Rule::SquareOpen], self.t0, Rule::Square, self.at());
            }
        }

    }

    enum Section {
        Dictionary { subsections: HashMap<Rc<str>, Section>, assign: ParsedDictionary },
        List { values: Vec<ParsedValue> },
        Value { value: ParsedValue },
    }

    fn dict_from_entries(entries: Vec<(Rc<str>, ParsedValue)>, at: Position) -> Result<HashMap<Rc<str>, ParsedValue>, ParseError> {
        let mut dict = HashMap::new();
        for (k, v) in entries.into_iter() {
            let pv = dict.insert(k.clone(), v.clone());
            if let Some(pv) = pv {
                return Err(ParseError::KeyAlreadyAssigned(at));
            }
        }
        Ok(dict)
    }

    fn make_sections(sections: Vec<(Vec<Rc<str>>, Section, bool)>, at: Position) -> Result<HashMap<Rc<str>, Section>, ParseError> {
        let mut dictionary = HashMap::new();
        for (key, section, append) in sections {
            assert!(key.len() > 0);
            let mut dictionary_reference: &mut HashMap<Rc<str>, Section> = &mut dictionary;
            let l = key.len() - 1;
            let mut i = 0;
            while i < l {
                let k = key[i].clone();
                if dictionary_reference.contains_key(k.deref()) {
                    let d = dictionary_reference.get_mut(k.deref()).unwrap();
                    match d {
                        Section::Dictionary { subsections, .. } => dictionary_reference = subsections,
                        _ => return Err(ParseError::SectionMultipleAssign()), // TODO
                    }
                } else {
                    dictionary_reference.insert(k.clone(), Section::Dictionary { subsections: HashMap::new(), assign: ParsedDictionary::empty() });
                    let d = dictionary_reference.get_mut(k.deref()).unwrap();
                    dictionary_reference = match d {
                        Section::Dictionary { subsections, .. } => subsections,
                        _ => unreachable!(),
                    };
                }
                i += 1;
            }
            let k = key[l].clone();
            if !append {
                if !dictionary_reference.contains_key(k.deref()) {
                    dictionary_reference.insert(k, section);
                } else {
                    return Err(ParseError::KeyAlreadyAssigned(at));
                }
            } else {
                if !dictionary_reference.contains_key(k.deref()) {
                    dictionary_reference.insert(k.clone(), Section::List { values: vec![] });
                }
                let list = dictionary_reference.get_mut(k.deref()).unwrap();
                if let Section::List { values } = list {
                    match section {
                        Section::Dictionary { assign, subsections } => {
                            assert!(subsections.is_empty());
                            values.push(ParsedValue::Dictionary(assign, at, at));
                        }
                        Section::List { values: es } => {
                            values.push(ParsedValue::List(ParsedList { elements: es }, at, at));
                        }
                        Section::Value { value } => {
                            values.push(value);
                        }
                    }
                } else {
                    unreachable!();
                }
            }
        }
        Ok(dictionary)
    }

    /// Construct a dictionary from entries and sections.
    fn join_sections(sections: HashMap<Rc<str>, Section>, assign: HashMap<Rc<str>, ParsedValue>, at: Position) -> Result<ParsedDictionary, ParseError> {
        let mut dict = ParsedDictionary::empty();
        for (sk, sv) in sections.into_iter() {
            match sv {
                Section::Dictionary { subsections, assign } => {
                    let subdict = join_sections(subsections, assign.entries, at)?;
                    dict.entries.insert(sk, ParsedValue::Dictionary(subdict, at, at));
                }
                Section::List { values } => {
                    dict.entries.insert(sk.clone(), ParsedValue::List(ParsedList { elements: values }, at, at));
                }
                Section::Value { value } => {
                    dict.entries.insert(sk, value);
                }
            }
        }
        for (ak, av) in assign.into_iter() {
            if dict.entries.contains_key(&ak) {
                return Err(ParseError::KeyAlreadyAssigned(av.from()));
            }
            dict.entries.insert(ak, av);
        }
        Ok(dict)
    }

    //// Errors

    impl Parser<'_> {

        fn require_whitespace_after(&mut self) {
            if !self.t0.has_whitespace_after() {
                self.errors.push(ParseError::ExpectedWhitespace(self.at()))
            }
        }

        fn require_no_whitespace_after(&mut self) {
            if self.t0.has_whitespace_after() {
                self.errors.push(ParseError::UnexpectedWhitespace(self.at()))//TODO at
            }
        }

        fn require_whitespace_before(&mut self) {
            if !self.whitespace_before {
                self.errors.push(ParseError::ExpectedWhitespace(self.at_last()))
            }
        }

        fn require_no_whitespace_before(&mut self) {
            if self.whitespace_before {
                self.errors.push(ParseError::UnexpectedWhitespace(self.at_last()))
            }
        }

        fn require_end(&mut self) {
            if self.t0.has_whitespace_after() {
                self.errors.push(ParseError::UnexpectedWhitespace(self.at()))//TODO at
            }
        }

    }

    impl Parser<'_> {

        fn recover_value() {

        }

    }

    type ParsedHeaderKey = Vec<Rc<str>>;

    type ParsedKey = Rc<str>;

    type ParsedEntry = (ParsedKey, ParsedValue);

    impl ParseError {

        fn expectation_error<T>(expected: &'static [Rule], found: Rule, found_at: Position, found_in: Rule, found_in_at: Position) -> Result<T, ParseError> {
            Err(ParseError::Expected(expected, found, found_at, found_in, found_in_at))
        }

        fn token_expectation_error<T>(expected: &'static [Rule], found: &Grouped, found_in: Rule, found_in_at: Position) -> Result<T, ParseError> {
            Self::expectation_error(expected, token_to_rule(found), found.at(), found_in, found_in_at)
        }

    }

    fn token_to_rule(token: &Grouped) -> Rule {
        match token {
            Grouped::String(..) => Rule::String,
            Grouped::Colon(..) => Rule::Colon,
            Grouped::Semicolon(..) => Rule::Semicolon,
            Grouped::Bar(..) => Rule::Bar,
            Grouped::Tilde(..) => Rule::Tilde,
            Grouped::RightAngle(..) => Rule::RightAngle,
            Grouped::CurlyBracket(..) => Rule::Bracket,
            Grouped::SquareBracket(..) => Rule::Square,
            Grouped::AngleBracket(..) => Rule::Tag,
            Grouped::CurlyHeader(..) => Rule::BracketHeader,
            Grouped::SquareHeader(..) => Rule::SquareHeader,
            Grouped::TaggedValueHeader(..) => Rule::TaggedValue,
            Grouped::AssignmentHeader(..) => Rule::Key,
            Grouped::End(..) => Rule::Close,
        }
    }

    //// Parsing errors

    #[derive(Clone, Debug)]
    pub enum Rule {
        String,
        List,
        Bar,
        TaggedValue,
        Block,
        Word,
        Key,
        Equals,
        Colon,
        Semicolon,
        Tilde,
        RightAngle,
        Bracket,
        Square,
        Tag,
        BracketHeader,
        SquareHeader,
        Close,
        Catenation,
        Tuple,
        Value,
        Argument,
        BracketedValue,
        BracketedDictionary,
        BracketedList,
        TaggedArguments,
        Arguments,
        TabularList,
        DelimitedList,
        AlignedList,
        TaggedList,
        Header,
        AbsoluteDictionary,
        Entry,
        Text,
        Transcription,
        TextBlock,
        Dictionary,
        Attribute,
        Name,
        AngularBracket,
        Attributes,
        SquareOpen,
        BracketOpen,
        Root,
        MappedKey,
        MapArrow,
    }

    #[derive(Clone)]
    pub enum ParseError {
        /// Tried to escape EOS.
        EscapingEndOfStream,
        /// Invalid escape character sequence at X.
        InvalidEscapeSequence(Position),
        /// Hash was followed by a character not allowed.
        IllegalHashSequence(Position),
        /// Text block at X was never closed.
        UnclosedTextBlock(Position),
        /// Invalid text block configuration at X.
        InvalidTextBlockConfiguration(Position),
        /// Mismatched closing X at Y in scope Z at W.
        MismatchedClose(Position, Rule, Position, Rule),
        /// Expected X but found Y at Z in W at V.
        Expected(&'static [Rule], Rule, Position, Rule, Position),
        /// Expected Columns? Todo
        ExpectedColumns(Position, usize, usize),
        /// Value at key not dictionary.
        KeyNotDictionary(Position),
        /// The key at X is already assigned a value.
        KeyAlreadyAssigned(Position),
        /// Expected whitespace at X between Y and Z in W at V.
        // ExpectedWhitespace(Position, Rule, Rule, Rule, Position),
        ExpectedWhitespace(Position),
        /// Unexpected whitespace at X between Y and Z in W at V.
        //UnexpectedWhitespace(Position, Rule, Rule, Rule, Position),
        UnexpectedWhitespace(Position),
        /// Attribute name at X must be a word but found Y.
        AttributeMustBeKey(Position, Rule),
        /// Tag name at X must be a word but found Y.
        TagNameMustBeKey(Position, Rule),
        /// Expected end but found X at Y.
        ExpectedEnd(Rule, Position),
        /// Illegal equals in attribute at X.
        AttributeIllegalEquals(Position),
        /// Illegal equals in argument at X.
        ArgumentIllegalEquals(Position),
        /// Assigning a value to a dictionary section.
        InnerAssignToDictionarySection,
        /// Cannot assign to the same section multiple times.
        SectionMultipleAssign(),
        CannotCloseOpenTranscription(Position),
    }

    impl Debug for ParseError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", error_to_string(self))
        }
    }

    pub fn error_to_string(error: &ParseError) -> String {
        match error {
            ParseError::EscapingEndOfStream => {
                format!("Escaping EOS.")
            }
            ParseError::ExpectedColumns(at, c, columns) => {
                format!("Expected {} columns but found {} at {}:{}.", columns, c, at.line, at.column)
            }
            ParseError::Expected(expected, found, at, _, _) => {
                format!("Expected {} but found ⟨{:?}⟩ at {}:{}.", list(expected), found, at.line, at.column)
            }
            ParseError::InvalidEscapeSequence(at) => {
                format!("Encountered unknown escape sequence at {}:{}.", at.line, at.column)
            }
            ParseError::IllegalHashSequence(at) => {
                format!("Encountered unknown hash sequence at {}:{}.", at.line, at.column)
            }
            ParseError::UnclosedTextBlock(at) => {
                format!("Unclosed text block at {}:{}", at.line, at.column)
            }
            ParseError::InvalidTextBlockConfiguration(at) => {
                format!("Encountered invalid configuration in text block at {}:{}", at.line, at.column)
            }
            ParseError::KeyNotDictionary(at) => {
                format!("Key at {}:{} not assigned a dictionary.", at.line, at.column)
            }
            ParseError::KeyAlreadyAssigned(at) => {
                format!("Key at {}:{} is already assigned a value.", at.line, at.column)
            }
            ParseError::ExpectedWhitespace(at) => {
                format!("Expected whitespace at {}:{} between {} and {} within {} at {}.", at.line, at.column, "-", "-", "-", "-") //TODO
            }
            ParseError::UnexpectedWhitespace(at) => {
                format!("Unexpected whitespace at {}:{} between {} and {} within {} at {}.", at.line, at.column, "-", "-", "-", "-") //TODO
            }
            ParseError::AttributeMustBeKey(at, found) => {
                format!("Attribute name at {}:{} must be a word or transcription but found {:?}.", at.line, at.column, found)
            }
            ParseError::TagNameMustBeKey(at, found) => {
                format!("Tag name at {}:{} must be a word or transcription but found {:?}.", at.line, at.column, found)
            }
            ParseError::MismatchedClose(at, _found , scope_at, scope_type) => {
                format!("Mismatched closing at {}:{} in scope {:?} at {}:{}.", at.line, at.column, scope_type, scope_at.line, scope_at.column)
            }
            ParseError::ExpectedEnd(found, at) => {
                format!("Expected end but found {:?} at {}:{}.", found, at.line, at.column)
            }
            ParseError::AttributeIllegalEquals(at) => {
                format!("Illegal equals found in attribute at {}:{}.", at.line, at.column)
            }
            ParseError::ArgumentIllegalEquals(at) => {
                format!("Illegal equals found in argument at {}:{}.", at.line, at.column)
            }
            ParseError::InnerAssignToDictionarySection => {
                format!("Section dictionary cannot assign to header keys.")
            }
            ParseError::SectionMultipleAssign() => {
                format!("Cannot assign to section multiple times.")
            }
            ParseError::CannotCloseOpenTranscription(at) => format!("Cannot close open transcription at {}:{}.", at.line, at.column),
        }
    }

    fn list(expected: &[Rule]) -> String {
        let mut str = String::new();
        let mut iter = expected.iter();
        if let Some(s) = iter.next() {
            str.push_str(&format!("⟨{:?}⟩", s));
        };
        while let Some(s) = iter.next() {
            str.push_str(&format!("⟨{:?}⟩", s));
        };
        str
    }

    impl ScopeType {

        pub(crate) fn to_rule(&self) -> Rule {
            match self {
                ScopeType::Open => Rule::Root,
                ScopeType::Curly => Rule::Bracket,
                ScopeType::Square => Rule::Square,
                ScopeType::Angle => Rule::AngularBracket,
            }
        }

    }

}

pub mod grouper {

    use std::slice::Iter;
    use crate::parse::lex::Token;
    use crate::parse::parse::parser::Rule;
    use crate::parse::pdm::Position;

    /// Reduced token.
    /// - Replace whitespace with whitespace_before bool.
    /// - Remove repeated tildes; only one tilde is retained.
    ///
    /// Extra reduction: If
    ///
    /// - there is not a colon before
    /// - there is a colon after
    /// - there is a space after the colon (for tags only)
    ///
    /// then the tokens are reduced accordingly:
    ///
    /// - CurlyBracket → CurlyHeader
    /// - SquareBracket → SquareHeader
    /// - AngleBracket → TaggedValueHeader
    /// - String → AssignmentHeader
    #[derive(PartialEq, Eq, Clone)]
    pub enum Grouped {
        /// from, to, space before, type, content
        String(Position, Position, bool, StringType, String),
        Colon(Position, bool),
        Semicolon(Position, bool),
        Bar(Position, bool),
        Tilde(Position, bool),
        RightAngle(Position, bool),
        CurlyBracket(Position, Position, bool, bool, Vec<Grouped>),
        SquareBracket(Position, Position, bool, bool, Vec<Grouped>),
        AngleBracket(Position, Position, bool, bool, Vec<Grouped>),
        CurlyHeader(Position, Position, bool, Vec<Grouped>),
        SquareHeader(Position, Position, bool, Vec<Grouped>),
        TaggedValueHeader(Position, Position, bool, Vec<Grouped>),
        AssignmentHeader(Position, Position, StringType, String),
        End(Position),
    }

    #[derive(Clone, Eq, PartialEq)]
    pub enum ScopeType {
        Open, Curly, Square, Angle,
    }

    #[derive(Clone, Eq, PartialEq)]
    pub enum StringType {
        Word, ClosedTranscription, UnclosedTranscription, TextBlock
    }

    impl Grouped {
        pub fn to_type(&self) -> Rule {
            match self {
                Grouped::String(..) => Rule::String,
                Grouped::Colon(..) => Rule::Colon,
                Grouped::Semicolon(..) => Rule::Semicolon,
                Grouped::Bar(..) => Rule::Bar,
                Grouped::Tilde(..) => Rule::Tilde,
                Grouped::RightAngle(..) => Rule::RightAngle,
                Grouped::CurlyBracket(..) => Rule::Bracket,
                Grouped::SquareBracket(..) => Rule::Square,
                Grouped::AngleBracket(..) => Rule::AngularBracket,
                Grouped::CurlyHeader(..) => Rule::BracketHeader,
                Grouped::SquareHeader(..) => Rule::SquareHeader,
                Grouped::TaggedValueHeader(..) => Rule::TaggedValue,
                Grouped::AssignmentHeader(..) => Rule::Key,
                Grouped::End(..) => Rule::Close,
            }
        }

        pub(crate) fn has_whitespace_after(&self) -> bool {
            match self {
                Grouped::String(_, _, wa, _, ..) => *wa,
                Grouped::Colon(_, wa) => *wa,
                Grouped::Semicolon(_, wa) => *wa,
                Grouped::Bar(_, wa) => *wa,
                Grouped::Tilde(_, wa) => false, // TODO: Do this to never get ws after tilde.
                Grouped::RightAngle(_, wa) => *wa,
                Grouped::SquareBracket(_, _, _, wa, _) => *wa,
                Grouped::CurlyBracket(_, _, _, wa, _) => *wa,
                Grouped::AngleBracket(_, _, _, wa, _) => *wa,
                Grouped::CurlyHeader(_, _, _, _) => false,
                Grouped::SquareHeader(_, _, _, _) => false,
                Grouped::TaggedValueHeader(_, _, _, _) => false,
                Grouped::AssignmentHeader(_, _, _, _) => false,
                Grouped::End(..) => false,

            }
        }

        fn from(&self) -> Position {
            match self {
                Grouped::String(from, _, _, _, ..) => *from,
                Grouped::Colon(at, _) => *at,
                Grouped::Semicolon(at, _) => *at,
                Grouped::Bar(at, _) => *at,
                Grouped::Tilde(at, _) => *at,
                Grouped::RightAngle(at, _) => *at,
                Grouped::SquareBracket(from, _, _, _, _) => *from,
                Grouped::CurlyBracket(from, _, _, _, _) => *from,
                Grouped::AngleBracket(from, _, _, _, _) => *from,
                Grouped::CurlyHeader(from, ..) => *from,
                Grouped::SquareHeader(from, ..) => *from,
                Grouped::TaggedValueHeader(from, ..) => *from,
                Grouped::AssignmentHeader(from, ..) => *from,
                Grouped::End(at) => *at,
            }
        }

        pub(crate) fn at(&self) -> Position {
            self.from()
        }

        pub(crate) fn to(&self) -> Position { //TODO these should be index + 1
            match self {
                Grouped::String(_, to, ..) => *to,
                Grouped::Colon(at, ..) => *at,
                Grouped::Semicolon(at, ..) => *at,
                Grouped::Bar(at, ..) => *at,
                Grouped::Tilde(at, ..) => *at,
                Grouped::RightAngle(at, ..) => *at,
                Grouped::SquareBracket(_, to, ..) => *to,
                Grouped::CurlyBracket(_, to, ..) => *to,
                Grouped::AngleBracket(_, to, ..) => *to,
                Grouped::CurlyHeader(_, to, ..) => *to,
                Grouped::SquareHeader(_, to, ..) => *to,
                Grouped::TaggedValueHeader(_, to, ..) => *to,
                Grouped::AssignmentHeader(_, to, ..) => *to,
                Grouped::End(at) => *at,
            }
        }

    }

    pub enum ReduceError {
        /// Found unexpected closing X at Y in Z scope at W.
        MismatchedClose(ScopeType, Position, ScopeType, Position)
    }

    pub struct Reducer<'a> {
        stream: Iter<'a, Token>,
        t: [&'a Token; 4],
        /// Previous token.
        previous: &'a Token,
    }

    impl<'a> Reducer<'a> {
        pub fn new(tokens: &'a Vec<Token>) -> Self {
            const P: Token = Token::End(Position { index: 0, line: 0, column: 0 });
            let mut r = Self {
                stream: tokens.iter(),
                t: [&P, &P, &P, &P],
                previous: &P,
            };
            r.shift(); r.shift(); r.shift(); r.shift();
            r
        }

        fn shift(&mut self) {
            self.previous = self.t[0];
            self.t[0] = self.t[1];
            self.t[1] = self.t[2];
            self.t[2] = self.t[3];
            self.t[3] = self.stream.next().unwrap_or(self.t[3]);
        }
    }

    impl Reducer<'_> {
        pub fn reduce(&mut self) -> Result<Vec<Grouped>, ReduceError> {
            self.reduce_scope(ScopeType::Open, self.t[0].at())
        }

        fn reduce_bracket(&mut self, scope_at: Position) -> Result<Vec<Grouped>, ReduceError> {
            self.reduce_scope(ScopeType::Curly, scope_at)
        }

        fn reduce_square(&mut self, scope_at: Position) -> Result<Vec<Grouped>, ReduceError> {
            self.reduce_scope(ScopeType::Square, scope_at)
        }

        fn reduce_angle(&mut self, scope_at: Position) -> Result<Vec<Grouped>, ReduceError> {
            self.reduce_scope(ScopeType::Angle, scope_at)
        }

        fn reduce_scope(&mut self, scope_type: ScopeType, scope_at: Position) -> Result<Vec<Grouped>, ReduceError> {
            let mut tokens = vec![];
            loop {
                match self.t[0].clone() {
                    Token::Whitespace(_) => {
                        self.shift();
                    }
                    Token::Word(at, string, _) | Token::ClosedTranscription(at, string) | Token::OpenTranscription(at, string) | Token::TextBlock(at, string) => {
                        let string_type = if matches!(self.t[0], Token::Word(..)) {
                            StringType::Word
                        } else if matches!(self.t[0], Token::ClosedTranscription(..)) {
                            StringType::ClosedTranscription
                        } else if matches!(self.t[0], Token::OpenTranscription(..)) {
                            StringType::UnclosedTranscription
                        } else {
                            StringType::TextBlock
                        };
                        let escaped = match self.t[0] {
                            Token::Word(_, _, unescaped) => {
                                unescaped.clone()
                            }
                            Token::ClosedTranscription(_, string) | Token::OpenTranscription(_, string) | Token::TextBlock(_, string) => {
                                let mut escapes = vec![];
                                for c in string.chars() {
                                    escapes.push(true);
                                }
                                escapes
                            }
                            _ => unreachable!(),
                        };
                        let colon_before = matches!(self.previous, Token::Colon(..));
                        self.shift();
                        let colon_after = matches!(self.t[0], Token::Colon(..));
                        let to = self.t[0].at();
                        if !colon_before && colon_after {
                            let header = Grouped::AssignmentHeader(at, to, string_type, string.clone());
                            tokens.push(header);
                        } else {
                            let whitespace_after = matches!(self.t[0], Token::Whitespace(..));
                            let word = Grouped::String(at, to, whitespace_after, string_type, string.clone());
                            tokens.push(word);
                        }
                    }
                    Token::Colon(at) => {
                        let following = self.t[1];
                        let word = Grouped::Colon(at, Self::is_whitespace(following));
                        tokens.push(word);
                        self.shift();
                    }
                    Token::Semicolon(at) => {
                        let following = self.t[1];
                        let word = Grouped::Semicolon(at, Self::is_whitespace(following));
                        tokens.push(word);
                        self.shift();
                    }
                    Token::Bar(at) => {
                        let following = self.t[1];
                        let word = Grouped::Bar(at, Self::is_whitespace(following));
                        tokens.push(word);
                        self.shift();
                    }
                    Token::Tilde(at) => {
                        let following = self.t[1];
                        if matches!(tokens.last(), Some(Grouped::Tilde(..))) { // Remove repeated tildes.
                            tokens.pop();
                        }
                        let word = Grouped::Tilde(at, Self::is_whitespace(following));
                        tokens.push(word);
                        self.shift();
                    }
                    Token::RightAngle(at) => {
                        if scope_type == ScopeType::Angle {
                            tokens.push(Grouped::End(at));
                            break;
                        } else {
                            // An angular scope can never contain a right angle.
                            // This lets us distinguish these cases.
                            let following = self.t[1];
                            let bullet = Grouped::RightAngle(at, Self::is_whitespace(following));
                            tokens.push(bullet);
                            self.shift();
                        }
                    }
                    Token::RightBracket(at) => {
                        if scope_type == ScopeType::Curly {
                            tokens.push(Grouped::End(at));
                            break;
                        } else {
                            return Err(ReduceError::MismatchedClose(ScopeType::Curly, at, scope_type, scope_at));
                        }
                    }
                    Token::RightSquare(at) => {
                        if scope_type == ScopeType::Square {
                            tokens.push(Grouped::End(at));
                            break;
                        } else {
                            return Err(ReduceError::MismatchedClose(ScopeType::Square, at, scope_type, scope_at));
                        }
                    }
                    Token::End(at) => {
                        if scope_type == ScopeType::Open {
                            tokens.push(Grouped::End(at));
                            break;
                        } else {
                            return Err(ReduceError::MismatchedClose(ScopeType::Open, at, scope_type, scope_at));
                        }
                    }
                    Token::LeftBracket(at) => {
                        let colon_before = matches!(self.previous, Token::Colon(..));
                        self.shift();
                        let initial_whitespace = matches!(self.t[0], Token::Whitespace(..));
                        let scope = self.reduce_bracket(at)?;
                        self.shift();
                        let colon_after = matches!(self.t[0], Token::Colon(..));
                        let to = self.t[0].at();
                        if !colon_before && colon_after {
                            let header = Grouped::CurlyHeader(at, to, initial_whitespace, scope);
                            tokens.push(header);
                        } else {
                            let whitespace_after = matches!(self.t[0], Token::Whitespace(..));
                            let bracket = Grouped::CurlyBracket(at, to, initial_whitespace, whitespace_after, scope);
                            tokens.push(bracket);
                        }
                    }
                    Token::LeftSquare(at) => {
                        let colon_before = matches!(self.previous, Token::Colon(..));
                        self.shift();
                        let initial_whitespace = matches!(self.t[0], Token::Whitespace(..));
                        let scope = self.reduce_square(at)?;
                        self.shift();
                        let colon_after = matches!(self.t[0], Token::Colon(..));
                        let to = self.t[0].at();
                        if !colon_before && colon_after {
                            let header = Grouped::SquareHeader(at, to, initial_whitespace, scope);
                            tokens.push(header);
                        } else {
                            let whitespace_after = matches!(self.t[0], Token::Whitespace(..));
                            let square = Grouped::SquareBracket(at, to, initial_whitespace, whitespace_after, scope);
                            tokens.push(square);
                        }
                    }
                    Token::LeftAngle(at) => {
                        let colon_before = matches!(self.previous, Token::Colon(..));
                        self.shift();
                        let initial_whitespace = matches!(self.t[0], Token::Whitespace(..));
                        let scope = self.reduce_angle(at)?;
                        self.shift();
                        let colon_after = matches!(self.t[0], Token::Colon(..));
                        let whitespace_after_after = matches!(self.t[1], Token::Whitespace(..));
                        let to = self.t[0].at();
                        if !colon_before && colon_after && whitespace_after_after {
                            let header = Grouped::TaggedValueHeader(at, to, initial_whitespace, scope);
                            tokens.push(header);
                        } else {
                            let whitespace_after = matches!(self.t[0], Token::Whitespace(..));
                            let square = Grouped::AngleBracket(at, to, initial_whitespace, whitespace_after, scope);
                            tokens.push(square);
                        }
                    }
                }
            }
            Ok(tokens)
        }

        fn is_whitespace(token: &Token) -> bool {
            matches!(token, Token::Whitespace(..))
        }

    }

}
