//! Khi parser reference implementation. Implements a tree-based parser.
//!
//! A document conforms to a value, dictionary or list. Use the corresponding
//! function to parse a document: [parse_value_str], [parse_dictionary_str] or
//! [parse_list_str].

// An O(n) predictive and recursive parser. Works in three stages: First, the
// input string is lexed and tokenized. Second, some tokens are reduced. Bracket
// groups are reduced to a single token, forming a token tree. Some tokens such
// as whitespace and tildes are removed and become attributes on the reduced
// tokens. In the final stage the parser operates on these reduced tokens to
// create a parsed document model (AST).

use std::collections::{HashSet};
use crate::lex::{lex, LexError};
use crate::parse::parser::{ParseError, Parser};
use crate::parse::reducer::{Reduced, ReduceError, Reducer};
use crate::pdm::{ParsedDictionary, ParsedList, ParsedValue, Position};

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
fn tokenize(document: &str) -> Result<Vec<Reduced>, ParseError> {
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

    use std::collections::{HashMap, HashSet};
    use std::fmt::{Debug, Formatter};
    use std::ops::Deref;
    use std::rc::Rc;
    use std::slice::Iter;
    use std::vec;
    use crate::{Dictionary, Value};
    use crate::parse::reducer::{Reduced, ScopeType, StringType};
    use crate::pdm::{ParsedAttribute, ParsedDictionary, ParsedList, ParsedTaggedValue, ParsedText, ParsedTuple, ParsedValue, Position};

    pub struct Parser<'a> {
        stream: Iter<'a, Reduced>,
        pub t0: &'a Reduced,
        t1: &'a Reduced,
        strings: &'a mut HashSet<Rc<str>>,
        errors: &'a mut Vec<ParseError>,
        whitespace_before: bool,
        last_position: Position,
    }

    impl<'a> Parser<'a> {
        pub fn new(
            tokens: &'a Vec<Reduced>,
            strings: &'a mut HashSet<Rc<str>>,
            errors: &'a mut Vec<ParseError>,
            whitespace_before: bool,
            open_position: Position,
        ) -> Self {
            const DEFAULT: Reduced = Reduced::End(Position { index: 0, line: 0, column: 0 });
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
            matches!(self.t0, Reduced::End(..))
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
            let value = if matches!(self.t0, Reduced::String(..) | Reduced::CurlyBracket(..) | Reduced::SquareBracket(..) | Reduced::AngleBracket(..) | Reduced::Tilde(..) | Reduced::Bar(..) | Reduced::TaggedValueHeader(..)) {
                self.parse_value()?
            } else {
                ParsedValue::Tuple(ParsedTuple::Unit, self.t0.at(), self.t1.at())
            };
            Ok(value)
        }

        /// Parse a dictionary document.
        ///
        /// ```text
        /// <dictionary-document> → *
        ///                       | *<dictionary>*
        /// ```
        pub(crate) fn parse_dictionary_document(&mut self) -> Result<ParsedDictionary, ParseError> {
            let dictionary = if matches!(self.t0, Reduced::AssignmentHeader(..) | Reduced::CurlyHeader(..) | Reduced::SquareHeader(..)) {
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
            let list = if matches!(self.t0, Reduced::String(..) | Reduced::CurlyBracket(..) | Reduced::SquareBracket(..) | Reduced::AngleBracket(..) | Reduced::Tilde(..) | Reduced::Bar(..) | Reduced::Bullet(..) | Reduced::TaggedValueHeader(..)) {
                self.parse_list()?
            } else {
                ParsedList::empty()
            };
            Ok(list)
        }

        /// Parse a value.
        ///
        /// ```text
        /// <value> → <inner-value>
        ///         | "|" <inner-value>
        ///         | <tagged-value>
        /// ```
        fn parse_value(&mut self) -> Result<ParsedValue, ParseError> {
            match self.t0 {
                Reduced::String(..) | Reduced::CurlyBracket(..) | Reduced::SquareBracket(..) | Reduced::AngleBracket(..) | Reduced::Tilde(..) => self.parse_inner_value(),
                Reduced::Bar(..) => {
                    self.shift();
                    self.parse_inner_value()
                }
                Reduced::TaggedValueHeader(..) => self.parse_tagged_value(),
                _ => return ParseError::token_expectation_error(&[Rule::InnerValue, Rule::BarredInnerValue, Rule::TaggedValue], self.t0, Rule::Value, self.t0.at()),
            }
        }

        /// Parse a block.
        ///
        /// ```text
        /// <block> → <term>
        ///         | <term> <block>
        ///         | "~"
        ///         | "~" <block>
        ///
        /// <term> → <text>
        ///        | <bracketed-value>
        ///        | <bracketed-dictionary>
        ///        | <bracketed-list>
        ///        | <tagged-arguments>
        /// ```
        fn parse_block(&mut self) -> Result<ParsedValue, ParseError> {
            let mut terms: Vec<ParsedValue> = vec![];
            let mut whitespace = vec![];
            let from = self.at();
            let mut space_before = false;
            loop {
                space_before = self.whitespace_before;
                match self.t0 {
                    Reduced::String(..) => {
                        let text = self.parse_text()?;
                        push_term(&mut terms, &mut whitespace, text, space_before);
                    }
                    Reduced::CurlyBracket(..) => {
                        let value = self.parse_bracketed_construct()?;
                        push_term(&mut terms, &mut whitespace, value, space_before);
                    }
                    Reduced::SquareBracket(..) => {
                        let value = self.parse_bracketed_list()?;
                        push_term(&mut terms, &mut whitespace, value, space_before);
                    },
                    Reduced::AngleBracket(..) => {
                        let value = self.parse_tagged_arguments()?;
                        push_term(&mut terms, &mut whitespace, value, space_before);
                    },
                    Reduced::Tilde(..) => {
                        self.shift();
                        space_before = false;
                    }
                    _ => break,
                }
            }
            let to = self.at();
            return Ok(ParsedValue::from_terms(from, to, terms, whitespace));
            fn push_term(terms: &mut Vec<ParsedValue>, whitespace: &mut Vec<bool>, component: ParsedValue, ws_before: bool) {
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

        /// Parse text.
        ///
        /// ```text
        /// <text>  → <string>
        ///         | <string> <text'>
        /// <text'> → <string>
        ///         | <string> <text'>
        ///         | "~" <text'>
        /// ```
        fn parse_text(&mut self) -> Result<ParsedValue, ParseError> {
            let mut text = String::new();
            let mut escapes = vec![];
            let mut space_before = false;
            let from = self.at();
            if !matches!(self.t0, Reduced::String(..)) {
                return ParseError::token_expectation_error(&[Rule::String], self.t0, Rule::Text, self.t0.at());
            }
            loop {
                match self.t0 {
                    Reduced::String(.., b, _, string, stresc) => {
                        if space_before {
                            text.push(' ');
                            escapes.push(false);
                        }
                        text.push_str(string);
                        escapes.extend(stresc);
                        space_before = *b;
                        self.shift();
                    }
                    Reduced::Tilde(..) => {
                        space_before = false;
                        self.shift();
                    }
                    _ => break,
                }
                if !(matches!(self.t0, Reduced::String(..)) || matches!(self.t0, Reduced::Tilde(..)) && matches!(self.t1, Reduced::String(..))) {
                    break;
                }
            }
            let to = self.at();
            let str = self.store_str(&text);
            let text = ParsedText { str, escapes };
            Ok(ParsedValue::Text(text, from, to))
        }

        /// Parse an inner value.
        ///
        /// ```text
        /// <inner-value> → <tuple-element>
        ///               | <tuple>
        ///
        /// <tuple> → <tuple-element> "|" <tuple-element>
        ///         | <tuple-element> "|" <tuple>
        ///
        /// <tuple-element> → <block>
        ///                 | <mapped-key>
        /// ```
        fn parse_inner_value(&mut self) -> Result<ParsedValue, ParseError> {
            let mut elements = vec![];
            let mut mapped_keys = vec![];
            let from = self.at();
            loop {
                if !matches!(self.t0, Reduced::String(..) | Reduced::AssignmentHeader(..)) || !matches!(self.t1, Reduced::Colon(..) | Reduced::MapArrow(..)) {
                    let element = self.parse_block()?;
                    elements.push(element);
                } else {
                    let mapped_key = self.parse_mapped_key()?;
                    mapped_keys.push(mapped_key);
                }
                if !matches!(self.t0, Reduced::Bar(..)) || !matches!(self.t1, Reduced::String(..) | Reduced::AssignmentHeader(..) | Reduced::CurlyBracket(..) | Reduced::SquareBracket(..) | Reduced::AngleBracket(..) | Reduced::Tilde(..)) {
                    break;
                }
                self.shift();
            }
            let to = self.at_last();
            if !mapped_keys.is_empty() {
                let dictionary = create_dictionary(vec![(vec![], mapped_keys)], &mut self.errors, from, to);
                elements.insert(0, ParsedValue::Dictionary(dictionary, from, to));
            }
            let inner_value = if elements.len() == 1 {
                elements.pop().unwrap()
            } else {
                ParsedValue::Tuple(ParsedTuple::Multiple(elements.into_boxed_slice()), from, to)
            };
            Ok(inner_value)
        }

        /// Parse a mapped key.
        ///
        /// ```text
        /// <mapped-key> → <key> "=>" <block>
        /// ```
        fn parse_mapped_key(&mut self) -> Result<(ParsedKey, ParsedValue), ParseError> {
            let from = self.t0.at();
            if !matches!(self.t0, Reduced::AssignmentHeader(..) | Reduced::String(..)) {
                return ParseError::token_expectation_error(&[Rule::Key], self.t0, Rule::MappedKey, from);
            }
            let key = self.parse_key()?;
            if !matches!(self.t0, Reduced::MapArrow(..)) {
                return ParseError::token_expectation_error(&[Rule::MapArrow], self.t0, Rule::MappedKey, from);
            }
            self.shift();
            let value = self.parse_block()?;
            Ok((key, value))
        }

        /// Parse a tagged value.
        ///
        /// ```text
        /// <tagged-value> → <tag>":"_<value>
        /// ```
        fn parse_tagged_value(&mut self) -> Result<ParsedValue, ParseError> {
            let from = self.at();
            let tag = self.parse_tag()?;
            if !matches!(self.t0, Reduced::Colon(..)) {
                return ParseError::token_expectation_error(&[Rule::Colon], self.t0, Rule::TaggedValue, from);
            };
            self.require_whitespace_after();
            self.shift();
            let value = self.parse_value()?;
            let to = self.at();
            let tagged_value = match tag {
                Some((name, attributes)) => {
                    ParsedValue::Tagged(ParsedTaggedValue {
                        name: self.store_str(&name), attributes, value: Box::new(value),
                    }, from, to)
                }
                None => {
                    if value.is_tuple() {
                        ParsedValue::Tuple(ParsedTuple::Single(Box::new(value)), from, to)
                    } else {
                        value
                    }
                }
            };
            Ok(tagged_value)
        }

        /// Parse a dictionary.
        ///
        /// ```text
        /// <dictionary> → <delimited-dictionary>
        ///              | <aligned-dictionary>
        ///              | <absolute-dictionary>
        ///
        /// <absolute-dictionary>  → <absolute-dictionary'>
        ///                        | <inner-dictionary>_<absolute-dictionary'>
        /// <absolute-dictionary'> → <section>
        ///                        | <section>_<absolute-dictionary>
        ///
        /// <section> → <square-header>":"
        ///           | <square-header>":"_<list>
        ///           | <curly-header>":"
        ///           | <curly-header>":"_<inner-dictionary>
        ///           | <curly-header>":"_<value>
        /// ```
        fn parse_dictionary(&mut self) -> Result<ParsedDictionary, ParseError> {
            let mut dictionary_sections = vec![];
            let mut direct_entries = vec![];
            let from = self.at();
            if matches!(self.t0, Reduced::AssignmentHeader(..)) {
                let mut entries = self.parse_inner_dictionary()?;
                direct_entries.append(&mut entries);
            }
            loop {
                match self.t0 {
                    Reduced::CurlyHeader(..) => {
                        let header = self.parse_header()?;
                        if !matches!(self.t0, Reduced::Colon(..)) {
                            return ParseError::token_expectation_error(&[Rule::Colon], self.t0, Rule::AbsoluteDictionary, from);
                        }
                        self.require_no_whitespace_before();
                        self.require_whitespace_after(); // TODO
                        self.shift();
                        let content_from = self.at();
                        match self.t0 {
                            Reduced::String(..) | Reduced::CurlyBracket(..) | Reduced::SquareBracket(..) | Reduced::AngleBracket(..) | Reduced::Tilde(..) | Reduced::Bar(..) | Reduced::TaggedValueHeader(..) => {
                                let value = self.parse_value()?;
                                direct_entries.push((header, value));
                            }
                            Reduced::AssignmentHeader(..) => {
                                let entries = self.parse_inner_dictionary()?;
                                dictionary_sections.push((header, entries))
                            }
                            _ => direct_entries.push((header, ParsedValue::Dictionary(ParsedDictionary::empty(), content_from, content_from))),
                        }
                    }
                    Reduced::SquareHeader(..) => {
                        let header = self.parse_header()?;
                        if !matches!(self.t0, Reduced::Colon(..)) {
                            return ParseError::token_expectation_error(&[Rule::Colon], self.t0, Rule::AbsoluteDictionary, from);
                        }
                        self.require_no_whitespace_before();
                        self.require_whitespace_after();
                        self.shift();
                        let table_from = self.at();
                        if matches!(self.t0, Reduced::String(..) | Reduced::CurlyBracket(..) | Reduced::SquareBracket(..) | Reduced::AngleBracket(..) | Reduced::Tilde(..) | Reduced::Bar(..) | Reduced::TaggedValueHeader(..)) {
                            let list = self.parse_list()?;
                            let table_to = self.at_last();
                            direct_entries.push((header, ParsedValue::List(list, table_from, table_to)))
                        } else {
                            direct_entries.push((header, ParsedValue::List(ParsedList::empty(), table_from, table_from)))
                        }
                    }
                    _ => break,
                }
            }
            let to = self.at_last();
            dictionary_sections.push((vec![], direct_entries));
            let dictionary = create_dictionary(dictionary_sections, self.errors, from, to);
            Ok(dictionary)
        }

        /// Parse a dictionary header.
        ///
        /// ```text
        /// <curly-header> → "{"<key>"}"
        ///
        /// <square-header> → "["<key>"]"
        /// ```
        fn parse_header(&mut self) -> Result<ParsedKey, ParseError> {
            let mut key = vec![];
            let at = self.at();
            match self.t0 {
                Reduced::CurlyHeader(_, ht, fw, scope) | Reduced::SquareHeader(_, ht, fw, scope) => {
                    self.shift();
                    let mut parser = Parser::new(&scope, self.strings, self.errors, *fw, *ht);
                    parser.require_no_whitespace_before();
                    key = parser.parse_key()?;
                    parser.require_end();
                }
                _ => return ParseError::token_expectation_error(&[Rule::Key], self.t0, Rule::Header, at),
            }
            Ok(key)
        }

        /// Parse an inner dictionary.
        ///
        /// ```text
        /// <inner-dictionary> → <delimited-dictionary>
        ///                    | <aligned-dictionary>
        /// ```
        fn parse_inner_dictionary(&mut self) -> Result<Vec<ParsedEntry>, ParseError> {
            let entry = self.parse_entry()?;
            let entries = match self.t0 {
                Reduced::Semicolon(..) => {
                    self.shift();
                    if matches!(self.t0, Reduced::AssignmentHeader(..)) {
                        self.parse_delimited_dictionary(vec![entry])?
                    } else {
                        vec![entry]
                    }
                }
                Reduced::AssignmentHeader(..) => {
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
                if !matches!(self.t0, Reduced::Semicolon(..)) {
                    break;
                }
                self.shift();
                if !matches!(self.t0, Reduced::AssignmentHeader(..)) {
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
                if !matches!(self.t0, Reduced::AssignmentHeader(..)) {
                    break;
                }
                self.require_whitespace_before();
            }
            Ok(entries)
        }

        /// Parse an entry.
        ///
        /// ```text
        /// <entry> → <key>":" <value>
        ///
        /// <key> → <string>
        ///       | <string>":"<key>
        /// ```
        fn parse_entry(&mut self) -> Result<ParsedEntry, ParseError> {
            let at = self.at();
            if let Reduced::AssignmentHeader(.., s) | Reduced::String(.., s, _) = self.t0 {
                let key = self.parse_entry_key()?;
                //if !matches!(self.t0, Reduced::Colon(..)) {
                //    return ParseError::token_expectation_error(&[Rule::Colon], self.t0, Rule::Key, at);
                //}
                //self.shift();
                let value = self.parse_value()?;
                Ok((key, value))
            } else {
                return ParseError::token_expectation_error(&[Rule::Key], self.t0, Rule::Entry, at);
            }
        }

        /// Parse a key and a colon.
        fn parse_entry_key(&mut self) -> Result<Vec<Rc<str>>, ParseError> {
            let mut key = vec![];
            loop {
                let s = match self.t0 {
                    Reduced::String(_, _, _, _, s, _) => s,
                    Reduced::AssignmentHeader(_, _, _, s) => s,
                    _ => return ParseError::token_expectation_error(&[Rule::String], self.t0, Rule::Key, self.t0.at()),
                };
                let k = self.store_str(s);
                key.push(k);
                self.shift();
                if !matches!(self.t0, Reduced::Colon(..)) {
                    return ParseError::token_expectation_error(&[Rule::Colon], self.t0, Rule::Key, self.t0.at());
                }
                self.require_no_whitespace_before();
                self.shift();
                if !matches!(self.t0, Reduced::String(..) | Reduced::AssignmentHeader(..)) || !matches!(self.t1, Reduced::Colon(..)) {
                    break;
                }
                self.require_no_whitespace_before();
            }
            Ok(key)
        }

        /// Parse a key.
        ///
        /// ```text
        /// <key> → <string>
        ///       | <string>":"<key>
        /// ```
        fn parse_key(&mut self) -> Result<Vec<Rc<str>>, ParseError> {
            let mut key = vec![];
            loop {
                match self.t0 {
                    Reduced::String(.., s, _) | Reduced::AssignmentHeader(.., s) => {
                        let k = self.store_str(s);
                        key.push(k);
                    }
                    _ => return ParseError::token_expectation_error(&[Rule::String], self.t0, Rule::Key, self.t0.at())
                }
                self.shift();
                if !matches!(self.t0, Reduced::Colon(..)) {
                    break;
                }
                self.require_no_whitespace_before();
                self.require_no_whitespace_after();
                self.shift();
            }
            Ok(key)
        }

        /// Parse a list.
        ///
        /// ```text
        /// <list> → <delimited-list>
        ///        | <aligned-list>
        ///        | <tabular-list>
        ///        | <tagged-list>
        /// ```
        fn parse_list(&mut self) -> Result<ParsedList, ParseError> {
            match self.t0 {
                Reduced::Bullet(..) => self.parse_aligned_list(vec![]),
                Reduced::Bar(..) => {
                    self.shift();
                    let value = self.parse_inner_value()?;
                    if matches!(self.t0, Reduced::Bar(..)) {
                        self.shift();
                        self.parse_tabular_list(vec![value])
                    } else if matches!(self.t0, Reduced::Semicolon(..)) {
                        self.shift();
                        self.parse_delimited_list(vec![value])
                        // TODO: Set from
                    } else {
                        Ok(ParsedList { elements: vec![value] })
                    }
                }
                Reduced::String(..) | Reduced::Tilde(..) | Reduced::CurlyBracket(..) | Reduced::SquareBracket(..) | Reduced::AngleBracket(..) => {
                    self.parse_delimited_list(vec![])
                }
                Reduced::TaggedValueHeader(..) => {
                    let value = self.parse_tagged_value()?;
                    if matches!(self.t0, Reduced::TaggedValueHeader(..)) {
                        self.parse_tagged_list(vec![value])
                    } else if matches!(self.t0, Reduced::Semicolon(..)) {
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
                let value = self.parse_value()?;
                elements.push(value);
                if !matches!(self.t0, Reduced::Semicolon(..)) {
                    break;
                }
                self.shift();
                if !matches!(self.t0, Reduced::String(..) | Reduced::Tilde(..) | Reduced::Bar(..) | Reduced::CurlyBracket(..) | Reduced::SquareBracket(..) | Reduced::AngleBracket(..) | Reduced::TaggedValueHeader(..)) {
                    break;
                }
            }
            let list = ParsedList { elements };
            Ok(list)
        }

        /// Parse an aligned list.
        ///
        /// ```text
        /// <aligned-list> → ">"_<value>
        ///                | ">"_<value>_<aligned-list>
        /// ```
        fn parse_aligned_list(&mut self, elements: Vec<ParsedValue>) -> Result<ParsedList, ParseError> {
            let mut elements = elements;
            let at = self.at(); // TODO: Might be earlier.
            if !matches!(self.t0, Reduced::Bullet(..)) {
                return ParseError::token_expectation_error(&[Rule::RightAngle], self.t0, Rule::AlignedList, at);
            }
            loop {
                self.require_whitespace_after();
                self.shift();
                let value = self.parse_value()?;
                elements.push(value);
                if !matches!(self.t0, Reduced::Bullet(..)) {
                    break;
                }
                self.require_whitespace_before();
            }
            Ok(ParsedList { elements })
        }

        /// Parse a tabular list.
        ///
        /// ```text
        /// <tabular-list> → "|" <inner-value> "|"
        ///                | "|" <inner-value> "|"_<tabular-list>
        /// ```
        fn parse_tabular_list(&mut self, elements: Vec<ParsedValue>) -> Result<ParsedList, ParseError> {
            let mut elements = elements;
            if !matches!(self.t0, Reduced::Bar(..)) {
                return ParseError::token_expectation_error(&[Rule::Bar], self.t0, Rule::TabularList, self.t0.at());
            }
            loop {
                self.shift();
                let value = self.parse_inner_value()?;
                elements.push(value);
                if !matches!(self.t0, Reduced::Bar(..)) {
                    return ParseError::token_expectation_error(&[Rule::Bar], self.t0, Rule::TabularList, self.t0.at());
                }
                self.shift(); // TODO Check whitespace
                if !matches!(self.t0, Reduced::Bar(..)) {
                    break;
                }
            }
            Ok(ParsedList { elements })
        }

        /// Parse a tagged list.
        ///
        /// ```text
        /// <tagged-list> → <tagged-value>
        ///               | <tagged-value>_<tagged-list>
        /// ```
        fn parse_tagged_list(&mut self, elements: Vec<ParsedValue>) -> Result<ParsedList, ParseError> {
            let mut elements = elements;
            loop {
                let value = self.parse_tagged_value()?;
                elements.push(value);
                if !matches!(self.t0, Reduced::TaggedValueHeader(..)) {
                    break;
                }
            }
            Ok(ParsedList { elements })
        }

        /// Parse tagged arguments.
        ///
        /// ```text
        /// <tagged-arguments> → <tag>
        ///                    | <tag><arguments>
        /// ```
        fn parse_tagged_arguments(&mut self) -> Result<ParsedValue, ParseError> {
            let from = self.at();
            let tag = self.parse_tag()?;
            let mut arguments = if matches!(self.t0, Reduced::Colon(..)) {
                self.parse_arguments()?
            } else {
                vec![]
            };
            let to = self.at();
            let value = if arguments.len() == 0 {
                ParsedValue::Tuple(ParsedTuple::Unit, from, to)
            } else if arguments.len() == 1 {
                let argument = arguments.pop().unwrap();
                if argument.is_tuple() {
                    ParsedValue::Tuple(ParsedTuple::Single(Box::new(argument)), from, to)
                } else {
                    argument
                }
            } else {
                ParsedValue::Tuple(ParsedTuple::Multiple(arguments.into_boxed_slice()), from, to)
            };
            let tagged_arguments = match tag {
                Some((name, attributes)) => {
                    let name = self.store_str(&name);
                    ParsedValue::Tagged(ParsedTaggedValue { name, attributes, value: Box::new(value) }, from, to)
                }
                None => {
                    value
                }
            };
            Ok(tagged_arguments)
        }

        /// Parse arguments.
        ///
        /// ```text
        /// <arguments> → ":"<argument>
        ///             | ":"<argument><arguments>
        ///
        /// <argument> → <string>
        ///            | <bracketed-value>
        ///            | <bracketed-dictionary>
        ///            | <bracketed-list>
        ///            | <tagged-arguments>
        /// ```
        fn parse_arguments(&mut self) -> Result<Vec<ParsedValue>, ParseError> {
            let mut arguments = vec![];
            if !matches!(self.t0, Reduced::Colon(..)) {
                return ParseError::token_expectation_error(&[Rule::Colon], self.t0, Rule::Arguments, self.t0.at());
            }
            loop {
                self.require_no_whitespace_after();
                self.shift();
                match self.t0 {
                    Reduced::String(.., s, escapes) => {
                        let from = self.at();
                        self.shift();
                        let to = self.at();
                        let str = self.store_str(s);
                        let text = ParsedValue::Text(ParsedText { str, escapes: escapes.clone() }, from, to);
                        arguments.push(text);
                    }
                    Reduced::CurlyBracket(..) => {
                        let value = self.parse_bracketed_construct()?;
                        arguments.push(value);
                    }
                    Reduced::SquareBracket(..) => {
                        let list = self.parse_bracketed_list()?;
                        arguments.push(list);
                    }
                    Reduced::AngleBracket(..) => {
                        let argument = self.parse_tagged_arguments()?;
                        arguments.push(argument);
                        break;
                    }
                    _ => return ParseError::token_expectation_error(&[Rule::String, Rule::BracketedValue, Rule::BracketedDictionary, Rule::BracketedList, Rule::TaggedArguments], self.t0, Rule::Argument, self.t0.at()),
                }
                if !matches!(self.t0, Reduced::Colon(..)) {
                    break;
                }
            }
            Ok(arguments)
        }

        /// Parse a tag.
        ///
        /// ```text
        /// <tag> → "<"<word>">"
        ///       | "<"<word>_<attributes> ">"
        ///       | "<"">"
        /// ```
        fn parse_tag(&mut self) -> Result<Option<(Rc<str>, Vec<ParsedAttribute>)>, ParseError> {
            if let Reduced::AngleBracket(from, _, fw, _, scope) | Reduced::TaggedValueHeader(from, _, fw, scope) = self.t0 {
                self.shift();
                let mut parser = Parser::new(scope, self.strings, self.errors, *fw, *from);
                parser.require_no_whitespace_before();
                if parser.is_end() {
                    return Ok(None);
                }
                let name = match parser.t0 {
                    Reduced::String(from, _, _, t, name, _) | Reduced::AssignmentHeader(from, _, t, name) => {
                        if *t != StringType::Word {
                            parser.errors.push(ParseError::TagNameMustBeWord(*from, parser.t0.to_type()));
                        }
                        parser.store_str(name)
                    }
                    _ => return ParseError::token_expectation_error(&[Rule::Name], &parser.t0, Rule::Tag, *from),
                };
                parser.shift();
                let attributes = if matches!(parser.t0, Reduced::String(..) | Reduced::AssignmentHeader(..)) {
                    parser.parse_attributes()?
                } else {
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
        ///
        /// <attribute> → <word>
        ///             | <word>":"<string>
        /// ```
        fn parse_attributes(&mut self) -> Result<Vec<ParsedAttribute>, ParseError> {
            let mut attributes = vec![];
            if !matches!(self.t0, Reduced::AssignmentHeader(..) | Reduced::String(..)) {
                return ParseError::token_expectation_error(&[Rule::Attribute], self.t0, Rule::Attributes, self.at());
            }
            loop {
                let key = match self.t0 {
                    Reduced::String(_, _, _, t, key, _) | Reduced::AssignmentHeader(_, _, t, key) => {
                        if *t != StringType::Word {
                            return Err(ParseError::AttributeMustBeWord(self.at(), self.t0.to_type()));
                        }
                        self.store_str(key)
                    }
                    _ => break,
                };
                self.shift();
                if matches!(self.t0, Reduced::Colon(..)) {
                    self.shift();
                    let value = self.parse_string()?;
                    attributes.push(ParsedAttribute(key, Some(value)));
                } else {
                    attributes.push(ParsedAttribute(key, None));
                }
            }
            Ok(attributes)
        }

        /// Parse bracketed construct.
        ///
        /// ```text
        /// <bracketed-value> → "{" <value> "}"
        ///
        /// <bracketed-dictionary> → "{" "}"
        ///                        | "{" <dictionary> "}"
        /// ```
        fn parse_bracketed_construct(&mut self) -> Result<ParsedValue, ParseError> {
            if let Reduced::CurlyBracket(from, to, wi, _, scope) = self.t0 {
                self.shift();
                let mut parser = Parser::new(scope, self.strings, self.errors, *wi, *to);
                let value = match parser.t0 {
                    Reduced::AssignmentHeader(..) | Reduced::CurlyHeader(..) | Reduced::SquareHeader(..) => {
                        let dictionary = parser.parse_dictionary()?;
                        ParsedValue::Dictionary(dictionary, *from, *to)
                    }
                    Reduced::String(..) | Reduced::CurlyBracket(..) | Reduced::SquareBracket(..) | Reduced::AngleBracket(..) | Reduced::Bar(..) | Reduced::Tilde(..) | Reduced::TaggedValueHeader(..) => {
                        parser.parse_value()?
                    }
                    Reduced::End(..) => {
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
        fn parse_bracketed_list(&mut self) -> Result<ParsedValue, ParseError> {
            if let Reduced::SquareBracket(from, to, fw, _, scope) = &self.t0 {
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

        /// Parse a string.
        ///
        /// ```text
        /// <string> → <word>
        ///          | <transcription>
        ///          | <text-block>
        /// ```
        fn parse_string(&mut self) -> Result<Rc<str>, ParseError> {
            match self.t0 {
                Reduced::String(.., text, _) => {
                    self.shift();
                    Ok(self.store_str(text))
                }
                _ => return ParseError::token_expectation_error(&[Rule::Word, Rule::Transcription, Rule::TextBlock], self.t0, Rule::String, self.at()),
            }
        }

    }

    /// Construct a dictionary from entries and sections.
    fn create_dictionary(sections: Vec<(Vec<Rc<str>>, Vec<ParsedEntry>)>, errors: &mut Vec<ParseError>, from: Position, to: Position) -> ParsedDictionary {
        let mut dictionary = ParsedDictionary { entries: HashMap::new() };
        for (section_key, entries) in sections {
            let dictionary_reference = match resolve_dictionary(&mut dictionary, &section_key, from) {
                Ok(r) => r,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            };
            for (entry_key, value) in entries {
                let dictionary_reference = match resolve_dictionary(dictionary_reference, &entry_key[0 .. entry_key.len() - 1], from) {
                    Ok(r) => r,
                    Err(e) => {
                        errors.push(e);
                        continue;
                    }
                };
                let k = &entry_key[entry_key.len() - 1];
                if dictionary_reference.entries.contains_key(k) {
                    errors.push(ParseError::KeyAlreadyAssigned(from)); //TODO from
                    continue;
                } else {
                    dictionary_reference.entries.insert(k.clone(), value);
                }
            }
        }
        dictionary
    }

    fn resolve_dictionary<'a>(root: &'a mut ParsedDictionary, key: &[Rc<str>], at: Position) -> Result<&'a mut ParsedDictionary, ParseError> {
        let mut dictionary_reference = root;
        for k in key {
            let entries = &mut dictionary_reference.entries;
            if entries.contains_key(k.deref()) {
                if let Some(ParsedValue::Dictionary(ref mut d, ..)) = entries.get_mut(k.deref()) {
                    dictionary_reference = d;
                } else {
                    return Err(ParseError::KeyNotDictionary(at)); // TODO
                }
            } else {
                let d = ParsedDictionary::empty();
                entries.insert(k.clone(), ParsedValue::Dictionary(d, at, at));
                let d = entries.get_mut(k.deref()).unwrap();
                dictionary_reference = d.as_mut_dictionary().unwrap();
            }
        }
        Ok(dictionary_reference)
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

    type ParsedKey = Vec<Rc<str>>;

    type ParsedEntry = (ParsedKey, ParsedValue);

    impl ParseError {

        fn expectation_error<T>(expected: &'static [Rule], found: Rule, found_at: Position, found_in: Rule, found_in_at: Position) -> Result<T, ParseError> {
            Err(ParseError::Expected(expected, found, found_at, found_in, found_in_at))
        }

        fn token_expectation_error<T>(expected: &'static [Rule], found: &Reduced, found_in: Rule, found_in_at: Position) -> Result<T, ParseError> {
            Self::expectation_error(expected, token_to_rule(found), found.at(), found_in, found_in_at)
        }

    }

    fn token_to_rule(token: &Reduced) -> Rule {
        match token {
            Reduced::String(..) => Rule::String,
            Reduced::Colon(..) => Rule::Colon,
            Reduced::Semicolon(..) => Rule::Semicolon,
            Reduced::Bar(..) => Rule::Bar,
            Reduced::Tilde(..) => Rule::Tilde,
            Reduced::MapArrow(..) => Rule::MapArrow,
            Reduced::Bullet(..) => Rule::RightAngle,
            Reduced::CurlyBracket(..) => Rule::Bracket,
            Reduced::SquareBracket(..) => Rule::Square,
            Reduced::AngleBracket(..) => Rule::Tag,
            Reduced::CurlyHeader(..) => Rule::BracketHeader,
            Reduced::SquareHeader(..) => Rule::SquareHeader,
            Reduced::TaggedValueHeader(..) => Rule::TaggedValue,
            Reduced::AssignmentHeader(..) => Rule::Key,
            Reduced::End(..) => Rule::Close,
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
        InnerValue,
        BarredInnerValue,
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
        AttributeMustBeWord(Position, Rule),
        /// Tag name at X must be a word but found Y.
        TagNameMustBeWord(Position, Rule),
        /// Expected end but found X at Y.
        ExpectedEnd(Rule, Position),
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
            ParseError::AttributeMustBeWord(at, found) => {
                format!("Attribute name at {}:{} must be a word but found {:?}.", at.line, at.column, found)
            }
            ParseError::TagNameMustBeWord(at, found) => {
                format!("Tag name at {}:{} must be a word but found {:?}.", at.line, at.column, found)
            }
            ParseError::MismatchedClose(at, _found , scope_at, scope_type) => {
                format!("Mismatched closing at {}:{} in scope {:?} at {}:{}.", at.line, at.column, scope_type, scope_at.line, scope_at.column)
            }
            ParseError::ExpectedEnd(found, at) => {
                format!("Expected end but found {:?} at {}:{}.", found, at.line, at.column)
            }
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

pub mod reducer {

    use std::slice::Iter;
    use crate::lex::Token;
    use crate::parse::parser::Rule;
    use crate::pdm::Position;

    /// Reduced token.
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
    pub enum Reduced {
        /// from, to, bool, type, content, escapes
        String(Position, Position, bool, StringType, String, Vec<bool>),
        Colon(Position, bool),
        Semicolon(Position, bool),
        Bar(Position, bool),
        Tilde(Position, bool),
        Bullet(Position, bool),
        MapArrow(Position, bool),
        CurlyBracket(Position, Position, bool, bool, Vec<Reduced>),
        SquareBracket(Position, Position, bool, bool, Vec<Reduced>),
        AngleBracket(Position, Position, bool, bool, Vec<Reduced>),
        CurlyHeader(Position, Position, bool, Vec<Reduced>),
        SquareHeader(Position, Position, bool, Vec<Reduced>),
        TaggedValueHeader(Position, Position, bool, Vec<Reduced>),
        AssignmentHeader(Position, Position, StringType, String),
        End(Position),
    }

    #[derive(Clone, Eq, PartialEq)]
    pub enum ScopeType {
        Open, Curly, Square, Angle,
    }

    #[derive(Clone, Eq, PartialEq)]
    pub enum StringType {
        Word, Transcription, TextBlock
    }

    impl Reduced {
        pub fn to_type(&self) -> Rule {
            match self {
                Reduced::String(..) => Rule::String,
                Reduced::Colon(..) => Rule::Colon,
                Reduced::Semicolon(..) => Rule::Semicolon,
                Reduced::Bar(..) => Rule::Bar,
                Reduced::Tilde(..) => Rule::Tilde,
                Reduced::Bullet(..) => Rule::RightAngle,
                Reduced::MapArrow(..) => Rule::MapArrow,
                Reduced::CurlyBracket(..) => Rule::Bracket,
                Reduced::SquareBracket(..) => Rule::Square,
                Reduced::AngleBracket(..) => Rule::AngularBracket,
                Reduced::CurlyHeader(..) => Rule::BracketHeader,
                Reduced::SquareHeader(..) => Rule::SquareHeader,
                Reduced::TaggedValueHeader(..) => Rule::TaggedValue,
                Reduced::AssignmentHeader(..) => Rule::Key,
                Reduced::End(..) => Rule::Close,
            }
        }

        pub(crate) fn has_whitespace_after(&self) -> bool {
            match self {
                Reduced::String(_, _, wa, _, ..) => *wa,
                Reduced::Colon(_, wa) => *wa,
                Reduced::Semicolon(_, wa) => *wa,
                Reduced::Bar(_, wa) => *wa,
                Reduced::Tilde(_, wa) => false, // TODO: Do this to never get ws after tilde.
                Reduced::Bullet(_, wa) => *wa,
                Reduced::MapArrow(_, wa) => *wa,
                Reduced::SquareBracket(_, _, _, wa, _) => *wa,
                Reduced::CurlyBracket(_, _, _, wa, _) => *wa,
                Reduced::AngleBracket(_, _, _, wa, _) => *wa,
                Reduced::CurlyHeader(_, _, _, _) => false,
                Reduced::SquareHeader(_, _, _, _) => false,
                Reduced::TaggedValueHeader(_, _, _, _) => false,
                Reduced::AssignmentHeader(_, _, _, _) => false,
                Reduced::End(..) => false,

            }
        }

        fn from(&self) -> Position {
            match self {
                Reduced::String(from, _, _, _, ..) => *from,
                Reduced::Colon(at, _) => *at,
                Reduced::Semicolon(at, _) => *at,
                Reduced::Bar(at, _) => *at,
                Reduced::Tilde(at, _) => *at,
                Reduced::Bullet(at, _) => *at,
                Reduced::MapArrow(at, _) => *at,
                Reduced::SquareBracket(from, _, _, _, _) => *from,
                Reduced::CurlyBracket(from, _, _, _, _) => *from,
                Reduced::AngleBracket(from, _, _, _, _) => *from,
                Reduced::CurlyHeader(from, ..) => *from,
                Reduced::SquareHeader(from, ..) => *from,
                Reduced::TaggedValueHeader(from, ..) => *from,
                Reduced::AssignmentHeader(from, ..) => *from,
                Reduced::End(at) => *at,
            }
        }

        pub(crate) fn at(&self) -> Position {
            self.from()
        }

        pub(crate) fn to(&self) -> Position { //TODO these should be index + 1
            match self {
                Reduced::String(_, to, ..) => *to,
                Reduced::Colon(at, ..) => *at,
                Reduced::Semicolon(at, ..) => *at,
                Reduced::Bar(at, ..) => *at,
                Reduced::Tilde(at, ..) => *at,
                Reduced::Bullet(at, ..) => *at,
                Reduced::MapArrow(at, ..) => *at,
                Reduced::SquareBracket(_, to, ..) => *to,
                Reduced::CurlyBracket(_, to, ..) => *to,
                Reduced::AngleBracket(_, to, ..) => *to,
                Reduced::CurlyHeader(_, to, ..) => *to,
                Reduced::SquareHeader(_, to, ..) => *to,
                Reduced::TaggedValueHeader(_, to, ..) => *to,
                Reduced::AssignmentHeader(_, to, ..) => *to,
                Reduced::End(at) => *at,
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
        pub fn reduce(&mut self) -> Result<Vec<Reduced>, ReduceError> {
            self.reduce_scope(ScopeType::Open, self.t[0].at())
        }

        fn reduce_bracket(&mut self, scope_at: Position) -> Result<Vec<Reduced>, ReduceError> {
            self.reduce_scope(ScopeType::Curly, scope_at)
        }

        fn reduce_square(&mut self, scope_at: Position) -> Result<Vec<Reduced>, ReduceError> {
            self.reduce_scope(ScopeType::Square, scope_at)
        }

        fn reduce_angle(&mut self, scope_at: Position) -> Result<Vec<Reduced>, ReduceError> {
            self.reduce_scope(ScopeType::Angle, scope_at)
        }

        fn reduce_scope(&mut self, scope_type: ScopeType, scope_at: Position) -> Result<Vec<Reduced>, ReduceError> {
            let mut tokens = vec![];
            loop {
                match self.t[0].clone() {
                    Token::Whitespace(_) => {
                        self.shift();
                    }
                    Token::Word(at, string, _) | Token::Transcription(at, string) | Token::TextBlock(at, string) => {
                        let string_type = if matches!(self.t[0], Token::Word(..)) {
                            StringType::Word
                        } else if matches!(self.t[0], Token::Transcription(..)) {
                            StringType::Transcription
                        } else {
                            StringType::TextBlock
                        };
                        let escaped = match self.t[0] {
                            Token::Word(_, _, unescaped) => {
                                unescaped.clone()
                            }
                            Token::Transcription(_, string) | Token::TextBlock(_, string) => {
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
                            let header = Reduced::AssignmentHeader(at, to, string_type, string.clone());
                            tokens.push(header);
                        } else {
                            let whitespace_after = matches!(self.t[0], Token::Whitespace(..));
                            let word = Reduced::String(at, to, whitespace_after, string_type, string.clone(), escaped);
                            tokens.push(word);
                        }
                    }
                    Token::Colon(at) => {
                        let following = self.t[1];
                        let word = Reduced::Colon(at, Self::is_whitespace(following));
                        tokens.push(word);
                        self.shift();
                    }
                    Token::Semicolon(at) => {
                        let following = self.t[1];
                        let word = Reduced::Semicolon(at, Self::is_whitespace(following));
                        tokens.push(word);
                        self.shift();
                    }
                    Token::Bar(at) => {
                        let following = self.t[1];
                        let word = Reduced::Bar(at, Self::is_whitespace(following));
                        tokens.push(word);
                        self.shift();
                    }
                    Token::Tilde(at) => {
                        let following = self.t[1];
                        let word = Reduced::Tilde(at, Self::is_whitespace(following));
                        tokens.push(word);
                        self.shift();
                    }
                    Token::DoubleArrow(at) => {
                        let following = self.t[1];
                        let arrow = Reduced::MapArrow(at, Self::is_whitespace(following));
                        tokens.push(arrow);
                        self.shift();
                    }
                    Token::RightAngle(at) => {
                        if scope_type == ScopeType::Angle {
                            tokens.push(Reduced::End(at));
                            break;
                        } else {
                            // An angular scope can never contain a right angle.
                            // This lets us distinguish these cases.
                            let following = self.t[1];
                            let bullet = Reduced::Bullet(at, Self::is_whitespace(following));
                            tokens.push(bullet);
                            self.shift();
                        }
                    }
                    Token::RightBracket(at) => {
                        if scope_type == ScopeType::Curly {
                            tokens.push(Reduced::End(at));
                            break;
                        } else {
                            return Err(ReduceError::MismatchedClose(ScopeType::Curly, at, scope_type, scope_at));
                        }
                    }
                    Token::RightSquare(at) => {
                        if scope_type == ScopeType::Square {
                            tokens.push(Reduced::End(at));
                            break;
                        } else {
                            return Err(ReduceError::MismatchedClose(ScopeType::Square, at, scope_type, scope_at));
                        }
                    }
                    Token::End(at) => {
                        if scope_type == ScopeType::Open {
                            tokens.push(Reduced::End(at));
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
                            let header = Reduced::CurlyHeader(at, to, initial_whitespace, scope);
                            tokens.push(header);
                        } else {
                            let whitespace_after = matches!(self.t[0], Token::Whitespace(..));
                            let bracket = Reduced::CurlyBracket(at, to, initial_whitespace, whitespace_after, scope);
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
                            let header = Reduced::SquareHeader(at, to, initial_whitespace, scope);
                            tokens.push(header);
                        } else {
                            let whitespace_after = matches!(self.t[0], Token::Whitespace(..));
                            let square = Reduced::SquareBracket(at, to, initial_whitespace, whitespace_after, scope);
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
                            let header = Reduced::TaggedValueHeader(at, to, initial_whitespace, scope);
                            tokens.push(header);
                        } else {
                            let whitespace_after = matches!(self.t[0], Token::Whitespace(..));
                            let square = Reduced::AngleBracket(at, to, initial_whitespace, whitespace_after, scope);
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
