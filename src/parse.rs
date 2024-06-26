//! Parsing of Khi documents.
//!
//! A document conforms to an expression, a sequence or a dictionary. Use the corresponding
//! function to parse a document: [parse_expression_str], [parse_dictionary_str] or
//! [parse_table_str].

use std::collections::{HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref};
use std::rc::Rc;
use std::slice::Iter;
use crate::{Dictionary, Tag, Value, Table, Text, Element, Compound, Attribute, AttributeValue, Entry, Tuple};
use crate::lex::{lex, LexError, Position, Token};

//// Parse

/// Parse an expression string.
pub fn parse_expression_str(document: &str) -> Result<ParsedValue> {
    let tokens = tokenize(document)?;
    let mut iter = TokenIter::new(tokens.iter());
    let mut strings = HashSet::new();
    let value = parse_expression_document(&mut iter, &mut strings)?;
    if !matches!(iter.t0, Token::End(..)) {
        return iter.expectation_error(&[TokenType::End]);
    };
    Ok(value)
}

/// Parse a dictionary string.
pub fn parse_dictionary_str(document: &str) -> Result<ParsedDictionary> {
    let tokens = tokenize(document)?;
    let mut iter = TokenIter::new(tokens.iter());
    let mut strings = HashSet::new();
    let dictionary = parse_dictionary_document(&mut iter, &mut strings)?;
    if !matches!(iter.t0, Token::End(..) ) {
        return iter.expectation_error(&[TokenType::End]);
    };
    Ok(dictionary)
}

/// Parse a table string.
pub fn parse_table_str(document: &str) -> Result<ParsedTable> {
    let tokens = tokenize(document)?;
    let mut iter = TokenIter::new(tokens.iter());
    let mut strings = HashSet::new();
    let table = parse_table_document(&mut iter, &mut strings)?;
    if !matches!(iter.t0, Token::End(..)) {
        return iter.expectation_error(&[TokenType::End]);
    };
    Ok(table)
}

/// Convert a Khi document to tokens.
fn tokenize(document: &str) -> Result<Vec<Token>> {
    let chars = document.chars();
    let tokens = match lex(chars) {
        Ok(tokens) => tokens,
        Err(error) => {
            return match error {
                LexError::EscapeEOS => Err(ParseError::EscapingEndOfStream),
                LexError::InvalidEscapeSequence(at) => Err(ParseError::InvalidEscapeSequence(at)),
                LexError::InvalidHashSequence(at) => Err(ParseError::IllegalHashSequence(at)),
                LexError::UnclosedTextBlock(at) => Err(ParseError::UnclosedTextBlock(at)),
                LexError::InvalidTextBlockConfiguration(at) => Err(ParseError::InvalidTextBlockConfiguration(at)),
                LexError::IllegalEscapeCharacter(at) => Err(ParseError::IllegalEscapeCharacter(at)),
                LexError::UnclosedColonOperator(at) => Err(ParseError::UnclosedColonOperator(at)),
            };
        }
    };
    Ok(tokens)
}

pub type Result<T> = std::result::Result<T, ParseError>;

//// Token iterator

/// A whitespace equivalent token iterator.
struct TokenIter<'a> {
    tokens: Iter<'a, Token>,
    t0: &'a Token,
    t1: &'a Token,
    t2: &'a Token,
    t3: &'a Token,
}

impl <'a> TokenIter<'a> {

    fn new(tokens: Iter<'a, Token>) -> Self {
        let default = &Token::End(Position { index: 0, line: 0, column: 0 });
        let mut iter = TokenIter { tokens, t0: default, t1: default, t2: default, t3: default };
        iter.next(); iter.next(); iter.next(); iter.next();
        iter
    }

    fn next(&mut self) {
        self.t0 = self.t1;
        self.t1 = self.t2;
        self.t2 = self.t3;
        loop {
            self.t3 = self.tokens.next().unwrap_or(self.t2);
            if !(self.t2.to_type() == TokenType::Whitespace && self.t3.to_type() == TokenType::Whitespace) {
                break;
            }
        }
    }

    fn skip_whitespace(&mut self) -> bool {
        let mut skipped = false;
        loop {
            if matches!(self.t0, Token::Whitespace(..)) {
                skipped = true;
                self.next();
            } else {
                break;
            };
        };
        skipped
    }

    fn at(&self) -> Position {
        self.t0.position()
    }

    fn peek_next_glyph_token(&mut self) -> &Token {
        match self.t0 {
            Token::Whitespace(..) => {
                self.skip_lookahead_whitespace();
                self.t1
            }
            token => token,
        }
    }

    fn skip_lookahead_whitespace(&mut self) {
        loop {
            if matches!(self.t1, Token::Whitespace(..)) {
                self.t1 = self.tokens.next().unwrap_or(self.t1);
            } else {
                break;
            }
        }
    }

    fn consume_next_glyph_token(&mut self) {
        self.skip_whitespace();
        self.next();
    }

    fn token_type(&self) -> TokenType {
        self.t0.to_type()
    }

    fn expectation_error<T>(&self, token_type: &'static[TokenType]) -> Result<T> {
        Err(ParseError::Expected(token_type, self.token_type(), self.at()))
    }

}

//// Parser

/// Parse an expression document.
///
/// Recognizes `<expression-document>`.
///
/// ```text
/// <expression-document> = *
///                       | *<value>*
/// ```
fn parse_expression_document(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedValue> {
    let from = iter.at();
    iter.skip_whitespace();
    let value = if matches_value(iter.t0) {
        let value = parse_value(iter, strings)?;
        iter.skip_whitespace();
        value
    } else {
        let to = iter.at();
        ParsedValue::nil(from, to)
    };
    Ok(value)
}

/// Parse a dictionary document.
///
/// Recognizes `<dictionary-document>`.
///
/// ```text
/// <dictionary-document> = *
///                       | *<dictionary>*
/// ```
fn parse_dictionary_document(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedDictionary> {
    iter.skip_whitespace();
    let dictionary = if matches!(iter.t0, Token::Word(..) | Token::Transcription(..) | Token::TextBlock(..) | Token::RightAngle(..)) {
        let dictionary = parse_dictionary(iter, strings)?;
        iter.skip_whitespace();
        dictionary
    } else {
        ParsedDictionary::empty()
    };
    Ok(dictionary)
}

/// Parse a table document.
///
/// Recognizes `<table-document>`.
///
/// ```text
/// <table-document> = *
///                  | *<table>*
/// ```
fn parse_table_document(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedTable> {
    iter.skip_whitespace();
    let table = if matches!(iter.t0, Token::Tilde(..) | Token::Word(..) | Token::Transcription(..) | Token::TextBlock(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..) | Token::Ampersand(..) | Token::Bar(..) | Token::RightAngle(..)) {
        let table = parse_table(iter, strings)?;
        iter.skip_whitespace();
        table
    } else {
        ParsedTable::empty()
    };
    Ok(table)
}

/// Parse a value.
///
/// Recognizes `<value>`.
///
/// ```text
/// <value> = <nested-expression>
///         | <tuple-expression>
///         | <expression>
///
/// <nested-expression> = <tag>":"_<value>
///                     | "<>"":"_<value>
///
/// <tuple-expression> = "&" <tuple-expression'>
///                    | <expression> "&" <tuple-expression'>
/// <tuple-expression'> = <expression>
///                     | <expression> "&" <tuple-expression'>
/// ```
fn parse_value(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedValue> {
    if !matches_value(iter.t0) {
        return tuple_expression_error(iter.t0, iter.at());
    }
    let from = iter.at();
    let mut argfrom = from;
    let mut tail = vec![];
    if matches!(iter.t0, Token::Ampersand(..)) {
        iter.next();
        if !matches_expression(iter.peek_next_glyph_token()) {
            let to = iter.at();
            return Ok(ParsedValue::Tuple(ParsedTuple::Unit, from, to));
        }
        iter.skip_whitespace();
        argfrom = iter.at();
        let e = parse_expression(iter, strings)?;
        tail.push(e);
    } else {
        let expression = parse_expression(iter, strings)?;
        match expression {
            ParsedValue::Tuple(t, from, to) => {
                if t.is_empty() && matches!(iter.t0, Token::Colon(..)) && matches!(iter.t1, Token::Whitespace(..)) {
                    iter.next(); iter.skip_whitespace();
                    let inner = parse_value(iter, strings)?;
                    let to = iter.at();
                    if inner.is_tuple() {
                        return Ok(ParsedValue::Tuple(ParsedTuple::Single(Box::new(inner)), from, to));
                    } else {
                        return Ok(inner);
                    }
                } else {
                    tail.push(ParsedValue::Tuple(t, from, to));
                }
            }
            ParsedValue::Tag(t, from, to) => {
                if t.value.is_unit() && matches!(iter.t0, Token::Colon(..)) && matches!(iter.t1, Token::Whitespace(..)) {
                    iter.next(); iter.skip_whitespace();
                    let inner = parse_value(iter, strings)?;
                    let to = iter.at();
                    return Ok(ParsedValue::Tag(ParsedTag { name: t.name, attributes: t.attributes, value: Box::new(inner) }, from, to));
                } else {
                    tail.push(ParsedValue::Tag(t, from, to));
                }
            }
            x => {
                tail.push(x);
            }
        }
    }
    loop {
        if !matches!(iter.peek_next_glyph_token(), Token::Ampersand(..)) {
            break;
        }
        iter.skip_whitespace();
        iter.next();
        if !matches_expression(iter.peek_next_glyph_token()) {
            return expression_error(iter.t0, iter.at());
        }
        iter.skip_whitespace();
        let e = parse_expression(iter, strings)?;
        tail.push(e);
    }
    let to = iter.at();
    Ok(ParsedValue::from_tuple(tail, from, to))
}

/// Parse an expression.
///
/// Recognizes `<expression>`.
///
/// ```text
/// <expression> = <word>
///              | <word> <expression>
///              | <transcription>
///              | <transcription> <expression>
///              | <text-block>
///              | <text-block> <expression>
///              | <bracketed-value>
///              | <bracketed-value> <expression>
///              | <bracketed-dictionary>
///              | <bracketed-dictionary> <expression>
///              | <bracketed-table>
///              | <bracketed-table> <expression>
///              | <tuple>
///              | <tuple> <expression>
///              | <tagged-value>
///              | <tagged-value> <expression>
///              | "~"
///              | "~" <expression>
/// ```
fn parse_expression(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedValue> {
    let mut terms: Vec<ParsedValue> = vec![];
    let mut whitespace = vec![];
    let from = iter.at();
    let mut after_whitespace = false;
    if !matches_expression(iter.t0) {
        return expression_error(iter.t0, iter.at());
    }
    loop {
        match iter.t0 {
            Token::Word(from, string) | Token::Transcription(from, string) | Token::TextBlock(from, string) => {
                let mut text = String::new();
                text.push_str(string);
                iter.next();
                let mut interspace = false;
                loop {
                    match iter.t0 {
                        Token::Word(_, string) | Token::Transcription(_, string) | Token::TextBlock(_, string) => {
                            iter.next();
                            if interspace {
                                text.push(' ');
                                interspace = false;
                            }
                            text.push_str(string);
                        }
                        Token::Whitespace(..) => {
                            if !matches!(iter.peek_next_glyph_token(), Token::Word(..) | Token::Transcription(..) | Token::TextBlock(..) | Token::Tilde(..)) {
                                break;
                            }
                            iter.skip_whitespace();
                            interspace = true;
                        }
                        Token::Tilde(..) => {
                            iter.next();
                            iter.skip_whitespace();
                            interspace = false;
                        }
                        _ => break,
                    }
                }
                let to = iter.at();
                let str = store_str(strings, &text);
                let text = ParsedText { str };
                let text = ParsedValue::Text(text, *from, to);
                push_term(&mut terms, &mut whitespace, &mut after_whitespace, text);
            }
            Token::LeftBracket(..) => push_term(&mut terms, &mut whitespace, &mut after_whitespace, parse_bracketed_structure(iter, strings)?),
            Token::Diamond(..) => push_term(&mut terms, &mut whitespace, &mut after_whitespace, parse_tuple(iter, strings)?),
            Token::LeftSquare(..) => push_term(&mut terms, &mut whitespace, &mut after_whitespace, parse_bracketed_table(iter, strings)?),
            Token::LeftAngle(..) => push_term(&mut terms, &mut whitespace, &mut after_whitespace, parse_tagged_value(iter, strings)?),
            Token::Whitespace(..) => {
                if !matches_expression(iter.peek_next_glyph_token()) {
                    break;
                }
                iter.skip_whitespace();
                after_whitespace = true;
            }
            Token::Tilde(..) => {
                iter.next();
                iter.skip_whitespace();
                after_whitespace = false;
            }
            _ => break,
        }
    };
    let to = iter.at();
    return Ok(ParsedValue::from_terms(from, to, terms, whitespace));
    fn push_term(terms: &mut Vec<ParsedValue>, whitespace: &mut Vec<bool>, after_whitespace: &mut bool, component: ParsedValue) {
        if terms.len() != 0 {
            if *after_whitespace {
                whitespace.push(true);
            } else {
                whitespace.push(false);
            };
            *after_whitespace = false;
        };
        terms.push(component);
    }
}

/// Parse a dictionary.
///
/// Recognizes `<dictionary>`.
///
/// ```text
/// <dictionary> = <flow-dictionary>
///              | <bullet-dictionary>
///
/// <dictionary-entry> = <word> ":" <value>
///                    | <transcription> ":" <value>
///                    | <text-block> ":" <value>
/// ```
fn parse_dictionary(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedDictionary> {
    match iter.t0 {
        Token::Word(..) | Token::Transcription(..) | Token::TextBlock(..) => parse_flow_dictionary(iter, strings),
        Token::RightAngle(..) => parse_bullet_dictionary(iter, strings),
        _ => iter.expectation_error(&[TokenType::Word, TokenType::Transcription, TokenType::TextBlock, TokenType::RightAngle]),
    }
}

/// Parse a flow dictionary.
///
/// Recognizes `<flow-dictionary>`.
///
/// ```text
/// <flow-dictionary> = <dictionary-entry>
///                   | <dictionary-entry> ";"
///                   | <dictionary-entry> ";" <flow-dictionary>
/// ```
fn parse_flow_dictionary(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedDictionary> {
    let mut entries = vec![];
    if !matches!(iter.t0, Token::Word(..) | Token::Transcription(..) | Token::TextBlock(..)) {
        return iter.expectation_error(&[TokenType::Word, TokenType::Transcription, TokenType::TextBlock]);
    };
    loop {
        let key = parse_text_token(iter, strings)?;
        if !matches!(iter.t0, Token::Colon(..)) {
            return iter.expectation_error(&[TokenType::Colon]);
        }
        iter.next();
        iter.skip_whitespace();
        let value = parse_value(iter, strings)?;
        entries.push(ParsedEntry(key, value));
        if !matches!(iter.peek_next_glyph_token(), Token::Semicolon(..)) {
            return Ok(ParsedDictionary { entries });
        };
        iter.consume_next_glyph_token();
        if !matches!(iter.peek_next_glyph_token(), Token::Word(..) | Token::Transcription(..) | Token::TextBlock(..)) {
            return Ok(ParsedDictionary { entries });
        };
        iter.skip_whitespace();
    };
}

/// Parse a bullet dictionary.
///
/// Recognizes `<bullet-dictionary>`.
///
/// ```text
/// <bullet-dictionary> = ">" <dictionary-entry>
///                     | ">" <dictionary-entry> <bullet-dictionary>
/// ```
fn parse_bullet_dictionary(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedDictionary> {
    let mut entries = vec![];
    if !matches!(iter.t0, Token::RightAngle(..)) {
        return iter.expectation_error(&[TokenType::RightAngle]);
    }
    loop {
        iter.next();
        iter.skip_whitespace();
        let key = parse_text_token(iter, strings)?;
        if !matches!(iter.t0, Token::Colon(..)) {
            return iter.expectation_error(&[TokenType::Colon]);
        }
        iter.next();
        iter.skip_whitespace();
        let value = parse_value(iter, strings)?;
        entries.push(ParsedEntry(key, value));
        if !matches!(iter.peek_next_glyph_token(), Token::RightAngle(..)) {
            return Ok(ParsedDictionary { entries });
        }
        iter.skip_whitespace();
    }
}

/// Parse a table.
///
/// Recognizes `<table>`.
///
/// ```text
/// ### All rows must have the same number of columns.
/// <table> = <flow-table>
///         | <grid-table>
///         | <bullet-table>
/// ```
fn parse_table(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedTable> {
    match iter.peek_next_glyph_token() {
        a if matches_value(a) => parse_flow_table(iter, strings),
        Token::Bar(..) => {
            let (entries, columns) = parse_grid_table(iter, strings)?;
            Ok(ParsedTable { elements: entries, columns })
        },
        Token::RightAngle(..) => {
            let (entries, columns) = parse_bullet_table(iter, strings)?;
            Ok(ParsedTable { elements: entries, columns })
        }
        Token::Whitespace(..) => unreachable!(),
        _ => {
            iter.skip_whitespace();
            iter.expectation_error(&[TokenType::Tilde, TokenType::Word, TokenType::Transcription, TokenType::TextBlock, TokenType::LeftBracket, TokenType::Diamond, TokenType::LeftSquare, TokenType::LeftAngle, TokenType::Ampersand, TokenType::Bar, TokenType::RightAngle])
        }
    }
}

/// Parse a flow table.
///
/// Recognizes `<flow-table>`.
///
/// ```text
/// <flow-table> = <value>
///              | <value> ";"
///              | <value> ";" <flow-table>
///              | <value> "|" <flow-table>
/// ```
fn parse_flow_table(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedTable> {
    let mut entries = vec![];
    let mut columns = 0;
    loop { // Read the first row, and determine number of columns.
        let entry = parse_value(iter, strings)?;
        entries.push(entry);
        columns += 1;
        match iter.peek_next_glyph_token() {
            Token::Semicolon(..) => {
                iter.consume_next_glyph_token();
                if matches_value(iter.peek_next_glyph_token()) {
                    iter.skip_whitespace();
                    break;
                } else {
                    return Ok(ParsedTable { elements: entries, columns });
                };
            }
            Token::Bar(..) => {
                iter.consume_next_glyph_token();
                iter.skip_whitespace();
            }
            _ => {
                return Ok(ParsedTable { elements: entries, columns });
            }
        }
    };
    loop {
        let mut c = 0;
        loop {
            let entry = parse_value(iter, strings)?;
            entries.push(entry);
            c += 1;
            match iter.peek_next_glyph_token() {
                Token::Semicolon(..) => {
                    iter.consume_next_glyph_token();
                    if c != columns {
                        return Err(ParseError::ExpectedColumns(iter.at(), c, columns));
                    };
                    if matches_value(iter.peek_next_glyph_token()) {
                        iter.skip_whitespace();
                        break;
                    } else {
                        return Ok(ParsedTable { elements: entries, columns });
                    };
                }
                Token::Bar(..) => {
                    iter.consume_next_glyph_token();
                    iter.skip_whitespace();
                },
                _ => {
                    if c != columns {
                        return Err(ParseError::ExpectedColumns(iter.at(), c, columns));
                    };
                    return Ok(ParsedTable { elements: entries, columns });
                }
            }
        };
    }
}

/// Parse a grid table.
///
/// Recognizes `<grid-table>`.
///
/// ```text
/// ### Cannot have zero columns.
/// <grid-table> = "|" <value> <grid-table>
///              | "|" <grid-table>
///              | "|"
/// ```
fn parse_grid_table(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<(Vec<ParsedValue>, usize)> {
    let mut entries = vec![];
    let mut columns = 0;
    loop { // Parse the first row.
        if !matches!(iter.t0, Token::Bar(..)) {
            return iter.expectation_error(&[TokenType::Bar]);
        };
        iter.next();
        match iter.peek_next_glyph_token() {
            x if matches_value(x) => {
                iter.skip_whitespace();
                let entry = parse_value(iter, strings)?;
                columns += 1;
                entries.push(entry);
                iter.skip_whitespace();
            }
            Token::Bar(..) => {
                iter.skip_whitespace();
                if columns == 0 {
                    return Err(ParseError::ZeroColumns(iter.at()));
                };
                break;
            }
            _ => {
                if columns == 0 {
                    return Err(ParseError::ZeroColumns(iter.at()));
                };
                return Ok((entries, columns));
            }
        }
    };
    loop { // Parse the remaining rows.
        let mut c = 0;
        loop {
            if !matches!(iter.t0, Token::Bar(..)) {
                return iter.expectation_error(&[TokenType::Bar]);
            };
            iter.next();
            match iter.peek_next_glyph_token() {
                x if matches_value(x) => {
                    iter.skip_whitespace();
                    let entry = parse_value(iter, strings)?;
                    c += 1;
                    entries.push(entry);
                    iter.skip_whitespace();
                }
                Token::Bar(..) => {
                    iter.skip_whitespace();
                    if c != columns {
                        return Err(ParseError::ExpectedColumns(iter.at(), c, columns));
                    };
                    break;
                }
                _ => {
                    if c != columns {
                        return Err(ParseError::ExpectedColumns(iter.at(), c, columns));
                    };
                    return Ok((entries, columns));
                }
            };
        };
    };
}

/// Parse a bullet table.
///
/// Recognizes `<bullet-table>`.
///
/// ```text
/// <bullet-table> = ">" <value>
///                | ">" <value> <bullet-table'>
/// <bullet-table'> = <bullet-table>
///                 | "|" <value>
///                 | "|" <value> <bullet-table'>
/// ```
fn parse_bullet_table(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<(Vec<ParsedValue>, usize)> {
    let mut entries = vec![];
    let mut columns = 1;
    if !matches!(iter.t0, Token::RightAngle(..)) {
        return iter.expectation_error(&[TokenType::RightAngle]);
    }
    iter.next();
    iter.skip_whitespace();
    let entry = parse_value(iter, strings)?;
    entries.push(entry);
    loop { // Read the first row.
        if !matches!(iter.peek_next_glyph_token(), Token::RightAngle(..) | Token::Bar(..)) {
            return Ok((entries, columns));
        }
        iter.skip_whitespace();
        match iter.t0 {
            Token::Bar(..) => {
                columns += 1;
                iter.next();
                iter.skip_whitespace();
                let entry = parse_value(iter, strings)?;
                entries.push(entry);
            }
            Token::RightAngle(..) => break,
            _ => unreachable!(),
        }
    }
    loop { // Read the remaining rows.
        let mut c = 1;
        iter.next();
        iter.skip_whitespace();
        let entry = parse_value(iter, strings)?;
        entries.push(entry);
        loop {
            if !matches!(iter.peek_next_glyph_token(), Token::RightAngle(..) | Token::Bar(..)) {
                return if columns == c {
                    Ok((entries, columns))
                } else {
                    Err(ParseError::ExpectedColumns(iter.at(), c, columns))
                }
            }
            iter.skip_whitespace();
            match iter.t0 {
                Token::Bar(..) => {
                    c += 1;
                    iter.next();
                    iter.skip_whitespace();
                    let entry = parse_value(iter, strings)?;
                    entries.push(entry);
                }
                Token::RightAngle(..) => {
                    if columns == c {
                        break;
                    } else {
                        return Err(ParseError::ExpectedColumns(iter.at(), c, columns));
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

/// Parse a tuple.
///
/// Recognizes `<tuple>`.
///
/// ```text
/// <tuple> = "<>"
///         | "<>"<arguments>
/// ```
fn parse_tuple(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedValue> {
    if !matches!(iter.t0, Token::Diamond(..)) {
        return iter.expectation_error(&[TokenType::Diamond]);
    }
    let from = iter.at();
    iter.next();
    let arguments = if matches!(iter.t0, Token::Colon(..)) {
        parse_arguments(iter, strings)?
    } else {
        vec![]
    };
    let to = iter.at();
    Ok(ParsedValue::from_tuple(arguments, from, to))
}

/// Parse a tag.
///
/// Recognizes `<tag>`.
///
/// ```text
/// ### Name cannot start with a hash sign.
/// <tag> = "<"<word>">"
///       | "<"<word> <attributes> ">"
/// ```
fn parse_tag(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<(Rc<str>, Vec<ParsedAttribute>)> {
    if !matches!(iter.t0, Token::LeftAngle(..)) {
        return iter.expectation_error(&[TokenType::LeftAngle]);
    };
    iter.next();
    let name = match iter.t0 {
        Token::Word(_, name) => name,
        _ => return iter.expectation_error(&[TokenType::Word]),
    };
    iter.next();
    let name = store_str(strings, name);
    if name.starts_with("#") { // Tag name cannot start with a hash sign.
        return Err(ParseError::TagHashName(iter.at()));
    }
    if matches!(iter.t0, Token::RightAngle(..)) {
        iter.next();
        return Ok((name, vec![]));
    }
    iter.skip_whitespace();
    let attributes = parse_attributes(iter, strings)?;
    iter.skip_whitespace();
    if !matches!(iter.t0, Token::RightAngle(..)) {
        return iter.expectation_error(&[TokenType::RightAngle]);
    };
    iter.next();
    Ok((name, attributes))
}

/// Parse attributes.
///
/// Recognizes `<attributes>`.
///
/// ```text
/// <attributes> = <word>
///              | <word> <attributes>
///              | <word>":" <word>
///              | <word>":" <word> <attributes>
///              | <word>":" <transcription>
///              | <word>":" <transcription> <attributes>
///              | <word>":" <text-block>
///              | <word>":" <text-block> <attributes>
/// ```
fn parse_attributes(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<Vec<ParsedAttribute>> {
    let mut attributes = vec![];
    if !matches!(iter.t0, Token::Word(..)) {
        return iter.expectation_error(&[TokenType::Word]);
    }
    loop {
        if let Token::Word(_, key) = iter.t0 {
            iter.next();
            let key = store_str(strings, key);
            match iter.t0 {
                Token::Colon(..) => {
                    iter.next();
                    iter.skip_whitespace();
                    let value = parse_text_token(iter, strings)?;
                    attributes.push(ParsedAttribute(key, Some(value)));
                }
                _ => {
                    attributes.push(ParsedAttribute(key, None));
                }
            };
            if !matches!(iter.peek_next_glyph_token(), Token::Word(..)) {
                return Ok(attributes);
            }
            iter.skip_whitespace();
        } else {
            unreachable!();
        }
    };
}

/// Parse a tag.
///
/// Recognizes `<tagged-value>`.
///
/// ```text
/// <tagged-value> = <tag>
///                | <tag><arguments>
/// ```
fn parse_tagged_value(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedValue> {
    let from = iter.at();
    let (name, attributes) = parse_tag(iter, strings)?;
    let arguments = if matches!(iter.t0, Token::Colon(..)) {
        parse_arguments(iter, strings)?
    } else {
        vec![]
    };
    let to = iter.at();
    let value = ParsedValue::from_tuple(arguments, from, to);
    Ok(ParsedValue::Tag(ParsedTag { name, attributes, value: Box::new(value) }, from, to))
}

/// Parse arguments.
///
/// Recognizes `<arguments>`.
///
/// ```text
/// <arguments> = ":"<word>
///             | ":"<word><arguments>
///             | ":"<transcription>
///             | ":"<transcription><arguments>
///             | ":"<text-block>
///             | ":"<text-block><arguments>
///             | ":"<bracketed-value>
///             | ":"<bracketed-value><arguments>
///             | ":"<bracketed-dictionary>
///             | ":"<bracketed-dictionary><arguments>
///             | ":"<bracketed-table>
///             | ":"<bracketed-table><arguments>
///             | ":""<>"
///             | ":""<>"<arguments>
///             | ":"<tag>
///             | ":"<tag><arguments>
///             | ":""&"":"<tuple>
///             | ":""&"":"<tagged-value>
/// ```
fn parse_arguments(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<Vec<ParsedValue>> {
    let mut arguments = vec![];
    if !matches!(iter.t0, Token::Colon(..)) {
        return iter.expectation_error(&[TokenType::Colon]);
    }
    if matches!(iter.t1, Token::Whitespace(..)) { // TODO: If this is the head of nested expression
        return Ok(arguments);
    }
    iter.next();
    loop {
        match iter.t0 {
            Token::Word(.., s) | Token::Transcription(.., s) | Token::TextBlock(.., s) => {
                let from = iter.at();
                iter.next();
                let to = iter.at();
                let str = store_str(strings, s);
                let text = ParsedValue::Text(ParsedText { str }, from, to);
                arguments.push(text);
            }
            Token::LeftBracket(..) => {
                let value = parse_bracketed_structure(iter, strings)?;
                arguments.push(value);
            }
            Token::LeftSquare(..) => {
                let table = parse_bracketed_table(iter, strings)?;
                arguments.push(table);
            }
            Token::Diamond(..) => {
                let from = iter.at();
                iter.next();
                let to = iter.at();
                arguments.push(ParsedValue::Tuple(ParsedTuple::Unit, from, to));
            }
            Token::LeftAngle(..) => {
                let from = iter.at();
                let (name, attributes) = parse_tag(iter, strings)?;
                let name = store_str(strings, &name);
                let to = iter.at();
                let tag = ParsedValue::Tag(ParsedTag { name, attributes, value: Box::new(ParsedValue::Tuple(ParsedTuple::Unit, to, to)) }, from, to);
                arguments.push(tag);
            }
            Token::Ampersand(..) => {
                iter.next();
                if !matches!(iter.t0, Token::Colon(..)) {
                    return iter.expectation_error(&[TokenType::Colon]);
                }
                iter.next();
                let inner = match iter.t0 {
                    Token::Diamond(_) => {
                        parse_tuple(iter, strings)?
                    }
                    Token::LeftAngle(_) => {
                        parse_tagged_value(iter, strings)?
                    }
                    _ => return iter.expectation_error(&[TokenType::Diamond, TokenType::LeftAngle]),
                };
                arguments.push(inner);
                break;
            }
            _ => return iter.expectation_error(&[TokenType::Word, TokenType::Transcription, TokenType::TextBlock, TokenType::LeftBracket, TokenType::LeftSquare, TokenType::Diamond, TokenType::LeftAngle, TokenType::Ampersand]),
        }
        match iter.t0 {
            Token::Colon(..) => {
                if matches!(iter.t1, Token::Whitespace(..)) { // TODO: If this is the head of nested expression
                    return Ok(arguments);
                }
                iter.next();
            },
            _ => break,
        }
    }
    Ok(arguments)
}

// /// Parse a composable.
// ///
// /// Recognizes `<composable>`.
// ///
// /// ```text
// /// <composable> = | <word>
// ///                | <transcription>
// ///                | <text-block>
// ///                | <bracketed-expression>
// ///                | <bracketed-dictionary>
// ///                | <bracketed-table>
// ///                | <tuple>
// ///                | <tagged-value>
// /// ```
// fn parse_composable(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedValue> {
//     match iter.t0 {
//         Token::Word(.., text) | Token::Transcription(.., text) | Token::TextBlock(.., text) => {
//             let from = iter.at();
//             iter.next();
//             let to = iter.at();
//             let str = store_str(strings, text);
//             Ok(ParsedValue::Text(ParsedText { str }, from, to))
//         }
//         Token::LeftBracket(..) => parse_bracketed_structure(iter, strings),
//         Token::LeftSquare(..) => parse_bracketed_table(iter, strings),
//         Token::Diamond(..) => parse_tuple(iter, strings),
//         Token::LeftAngle(..) => parse_tagged_value(iter, strings),
//         _ => return iter.expectation_error(&[TokenType::Word, TokenType::Transcription, TokenType::TextBlock, TokenType::LeftBracket, TokenType::LeftSquare, TokenType::Diamond, TokenType::LeftAngle]),
//     }
// }

/// Parse a bracketed expression or dictionary.
///
/// Recognizes `<bracketed-value>` and `<bracketed-dictionary>`.
///
/// ```text
/// <bracketed-value> = "{" <value> "}"
///
/// <bracketed-dictionary> = "{" "}"
///                        | "{" <dictionary> "}"
/// ```
fn parse_bracketed_structure(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedValue> {
    if !matches!(iter.t0, Token::LeftBracket(..)) {
        return iter.expectation_error(&[TokenType::LeftBracket]);
    };
    let from = iter.at();
    iter.next();
    iter.skip_whitespace();
    match iter.t0 {
        Token::Word(..) | Token::Transcription(..) | Token::TextBlock(..) => {
            match iter.t1 {
                Token::Colon(..) => {
                    let dictionary = parse_dictionary(iter, strings)?;
                    iter.skip_whitespace();
                    if !matches!(iter.t0, Token::RightBracket(..)) {
                        return iter.expectation_error(&[TokenType::RightBracket]);
                    }
                    iter.next();
                    let to = iter.at();
                    Ok(ParsedValue::Dictionary(dictionary, from, to))
                }
                x if matches_expression(x) || matches!(x, Token::Whitespace(..) | Token::RightBracket(..)) => {
                    let value = parse_value(iter, strings);
                    iter.skip_whitespace();
                    if !matches!(iter.t0, Token::RightBracket(..)) {
                        return iter.expectation_error(&[TokenType::RightBracket]);
                    }
                    iter.next();
                    value
                }
                _ => {
                    iter.next();
                    return iter.expectation_error(&[TokenType::Colon, TokenType::Whitespace, TokenType::Word, TokenType::Transcription, TokenType::TextBlock, TokenType::LeftBracket, TokenType::Diamond, TokenType::LeftSquare, TokenType::LeftAngle, TokenType::Ampersand, TokenType::RightBracket]);
                }
            }
        }
       Token::Tilde(..) | Token::LeftBracket(..) | Token::Diamond(..) | Token::LeftSquare(..) | Token::LeftAngle(..) | Token::Ampersand(..) => { // Expression
            let value = parse_value(iter, strings)?;
            iter.skip_whitespace();
            if !matches!(iter.t0, Token::RightBracket(..)) {
                return iter.expectation_error(&[TokenType::RightBracket]);
            };
            iter.next();
            Ok(value)
        }
        Token::RightBracket(..) => { // Empty dictionary
            let dictionary = ParsedDictionary::empty();
            iter.next();
            let to = iter.at();
            Ok(ParsedValue::Dictionary(dictionary, from, to))
        }
        Token::RightAngle(..) => { // Bullet dictionary
            let dictionary = parse_dictionary(iter, strings)?;
            iter.skip_whitespace();
            if !matches!(iter.t0, Token::RightBracket(..)) {
                return iter.expectation_error(&[TokenType::RightBracket]);
            }
            iter.next();
            let to = iter.at();
            Ok(ParsedValue::Dictionary(dictionary, from, to))
        }
        Token::Whitespace(..) => unreachable!(),
        _ => return iter.expectation_error(&[TokenType::Word, TokenType::Transcription, TokenType::TextBlock, TokenType::Tilde, TokenType::LeftBracket, TokenType::LeftSquare, TokenType::LeftAngle, TokenType::Ampersand, TokenType::RightBracket, TokenType::RightAngle]),
    }
}

/// Parse a bracketed table.
///
/// Recognizes `<bracketed-table>`.
///
/// ```text
/// <bracketed-table> = "[" "]"
///                   | "[" <table> "]"
/// ```
fn parse_bracketed_table(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedValue> {
    let from = iter.at();
    if !matches!(iter.t0, Token::LeftSquare(..)) {
        return iter.expectation_error(&[TokenType::LeftSquare]);
    };
    iter.next();
    iter.skip_whitespace();
    match iter.t0 {
        x if matches_value(x) || matches!(x, Token::Bar(..) | Token::RightAngle(..)) => {
            let table = parse_table(iter, strings)?;
            iter.skip_whitespace();
            if !matches!(iter.t0, Token::RightSquare(..)) {
                return iter.expectation_error(&[TokenType::RightSquare]);
            };
            iter.next();
            let to = iter.at();
            Ok(ParsedValue::Table(table, from, to))
        }
        Token::RightSquare(..) => {
            iter.next();
            let to = iter.at();
            let table = ParsedTable::empty();
            Ok(ParsedValue::Table(table, from, to))
        }
        Token::Whitespace(..) => unreachable!(),
        _ => iter.expectation_error(&[TokenType::Tilde, TokenType::Word, TokenType::Transcription, TokenType::TextBlock, TokenType::LeftBracket, TokenType::Diamond, TokenType::LeftSquare, TokenType::LeftAngle, TokenType::Ampersand, TokenType::Bar, TokenType::RightAngle, TokenType::RightSquare]),
    }
}

//// Utility

fn matches_value(token: &Token) -> bool {
    matches_expression(token) || matches!(token, Token::Ampersand(..))
}

fn tuple_expression_error(token: &Token, at: Position) -> Result<ParsedValue> {
    Err(ParseError::Expected(&[TokenType::Word, TokenType::Transcription, TokenType::TextBlock, TokenType::LeftBracket, TokenType::LeftSquare, TokenType::Diamond, TokenType::LeftAngle, TokenType::Ampersand, TokenType::Tilde], token.to_type(), at))
}

fn matches_expression(token: &Token) -> bool {
    matches!(token, Token::Word(..) | Token::Transcription(..) | Token::TextBlock(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::Diamond(..) | Token::LeftAngle(..) | Token::Tilde(..))
}

fn expression_error(token: &Token, at: Position) -> Result<ParsedValue> {
    Err(ParseError::Expected(&[TokenType::Word, TokenType::Transcription, TokenType::TextBlock, TokenType::LeftBracket, TokenType::LeftSquare, TokenType::Diamond, TokenType::LeftAngle, TokenType::Tilde], token.to_type(), at))
}

/// Parse a word, transcription or text block.
///
/// Recognizes `<word>`, `<transcription>` and `<text-block>`.
fn parse_text_token(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<Rc<str>> {
    match iter.t0 {
        Token::Word(_, text) | Token::Transcription(_, text) | Token::TextBlock(_, text) => {
            iter.next();
            Ok(store_str(strings, text))
        }
        _ => iter.expectation_error(&[TokenType::Word, TokenType::Transcription, TokenType::TextBlock]),
    }
}

fn store_str(strings: &mut HashSet<Rc<str>>, candidate: &str) -> Rc<str> {
    if let Some(str) = strings.get(candidate) {
        str.clone()
    } else {
        let count = Rc::from(candidate);
        let str = Rc::clone(&count);
        strings.insert(count);
        str
    }
}

//// Possible parsing errors

#[derive(Clone)]
pub enum ParseError {
    /// Tried to escape EOS.
    EscapingEndOfStream,
    UnclosedTranscription(Position),
    ZeroColumns(Position),
    ExpectedColumns(Position, usize, usize),
    /// Expected X but found Y at Z.
    Expected(&'static[TokenType], TokenType, Position),
    InvalidEscapeSequence(Position),
    IllegalHashSequence(Position),
    TagHashName(Position),
    UnclosedTextBlock(Position),
    InvalidTextBlockConfiguration(Position),
    IllegalEscapeCharacter(Position),
    UnclosedColonOperator(Position),
    InvalidNestedExpression(Position),
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
        ParseError::UnclosedTranscription(at) => {
            format!("Unclosed transcription at {}:{}.", at.line, at.column)
        }
        ParseError::ZeroColumns(at) => {
            format!("Row with zero columns at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedColumns(at, c, columns) => {
            format!("Expected {} columns but found {} at {}:{}.", columns, c, at.line, at.column)
        }
        ParseError::Expected(expected, found, at) => {
            format!("Expected {} but found ⟨{}⟩ at {}:{}.", list(expected), found, at.line, at.column)
        }
        ParseError::InvalidEscapeSequence(at) => {
            format!("Encountered unknown escape sequence at {}:{}.", at.line, at.column)
        }
        ParseError::IllegalHashSequence(at) => {
            format!("Encountered unknown hash sequence at {}:{}.", at.line, at.column)
        }
        ParseError::TagHashName(at) => {
            format!("Tag name cannot start with hash sign at {}:{}", at.line, at.column)
        }
        ParseError::UnclosedTextBlock(at) => {
            format!("Unclosed text block at {}:{}", at.line, at.column)
        }
        ParseError::InvalidTextBlockConfiguration(at) => {
            format!("Encountered invalid configuration in text block at {}:{}", at.line, at.column)
        }
        ParseError::IllegalEscapeCharacter(at) => {
            format!("Illegal escape character at {}:{}", at.line, at.column)
        }
        ParseError::UnclosedColonOperator(at) => {
            format!("Unclosed colon operator at {}:{}", at.line, at.column)
        }
        ParseError::InvalidNestedExpression(at) => {
            format!("Invalid nested expression at {}:{}", at.line, at.column)
        }
    }
}

fn list(expected: &[TokenType]) -> String {
    let mut str = String::new();
    let mut iter = expected.iter();
    if let Some(s) = iter.next() {
        str.push_str(&format!("⟨{}⟩", s));
    };
    while let Some(s) = iter.next() {
        str.push_str(&format!("⟨{}⟩", s));
    };
    str
}

//// Parsing results

//// Value

/// A parsed value.
#[derive(Clone)]
pub enum ParsedValue {
    Nil(Position, Position),
    Text(ParsedText, Position, Position),
    Dictionary(ParsedDictionary, Position, Position),
    Table(ParsedTable, Position, Position),
    Compound(ParsedCompound, Position, Position),
    Tuple(ParsedTuple, Position, Position),
    Tag(ParsedTag, Position, Position),
}

impl ParsedValue {

    pub fn nil(from: Position, to: Position) -> Self {
        ParsedValue::Nil(from, to)
    }

    pub fn from_terms(from: Position, to: Position, mut terms: Vec<ParsedValue>, whitespace: Vec<bool>) -> Self {
        let len = terms.len();
        if len == 0 {
            ParsedValue::Nil(from, to)
        } else if len == 1 {
            terms.pop().unwrap()
        } else {
            assert_eq!(len - 1, whitespace.len());
            ParsedValue::Compound(ParsedCompound { components: terms, whitespace }, from, to)
        }
    }

    pub fn from(&self) -> Position {
        match self {
            ParsedValue::Nil(.., from, _) => from,
            ParsedValue::Text(.., from, _) => from,
            ParsedValue::Dictionary(.., from, _) => from,
            ParsedValue::Table(.., from, _) => from,
            ParsedValue::Compound(.., from, _) => from,
            ParsedValue::Tuple(.., from, _) => from,
            ParsedValue::Tag(.., from, _) => from,
        }.clone()
    }

    pub fn to(&self) -> Position {
        match self {
            ParsedValue::Nil(.., to) => to,
            ParsedValue::Text(.., to) => to,
            ParsedValue::Dictionary(.., to) => to,
            ParsedValue::Table(.., to) => to,
            ParsedValue::Compound(.., to) => to,
            ParsedValue::Tuple(.., to) => to,
            ParsedValue::Tag(.., to) => to,
        }.clone()
    }

    pub fn from_tuple(mut values: Vec<ParsedValue>, from: Position, to: Position) -> Self {
        let len = values.len();
        if len == 0 {
            ParsedValue::Tuple(ParsedTuple::Unit, from, to)
        } else if len == 1 {
            let value = values.remove(0);
            if matches!(value, ParsedValue::Tuple(..)) {
                ParsedValue::Tuple(ParsedTuple::Single(Box::new(value)), from, to)
            } else {
                value
            }
        } else {
            ParsedValue::Tuple(ParsedTuple::Multiple(values.into_boxed_slice()), from, to)
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, ParsedValue::Tuple(ParsedTuple::Unit, ..))
    }

}

impl Value<ParsedValue, ParsedText, ParsedDictionary, ParsedTable, ParsedCompound, ParsedTuple, ParsedTag> for ParsedValue {

    fn as_text(&self) -> Option<&ParsedText> {
        if let ParsedValue::Text(t, ..) = self {
            Some(t)
        } else {
            None
        }
    }

    fn as_dictionary(&self) -> Option<&ParsedDictionary> {
        if let ParsedValue::Dictionary(d, ..) = self {
            Some(d)
        } else {
            None
        }
    }

    fn as_tuple(&self) -> Option<&ParsedTuple> {
        if let ParsedValue::Tuple(d, _, _) = self {
            Some(d)
        } else {
            None
        }
    }

    fn as_table(&self) -> Option<&ParsedTable> {
        if let ParsedValue::Table(t, ..) = self {
            Some(t)
        } else {
            None
        }
    }

    fn as_compound(&self) -> Option<&ParsedCompound> {
        if let ParsedValue::Compound(d, ..) = self {
            Some(d)
        } else {
            None
        }
    }

    fn as_tag(&self) -> Option<&ParsedTag> {
        if let ParsedValue::Tag(d, ..) = self {
            Some(d)
        } else {
            None
        }
    }

    fn is_nil(&self) -> bool {
        matches!(self, ParsedValue::Nil(..))
    }

    fn is_text(&self) -> bool {
        matches!(self, ParsedValue::Text(..))
    }

    fn is_dictionary(&self) -> bool {
        matches!(self, ParsedValue::Dictionary(..))
    }

    fn is_tuple(&self) -> bool {
        matches!(self, ParsedValue::Tuple(..))
    }

    fn is_table(&self) -> bool {
        matches!(self, ParsedValue::Table(..))
    }

    fn is_compound(&self) -> bool {
        matches!(self, ParsedValue::Compound(..))
    }

    fn is_tag(&self) -> bool {
        matches!(self, ParsedValue::Tag(..))
    }

    fn unfold(&self) -> Box<[&ParsedValue]> {
        match self {
            ParsedValue::Tuple(tuple, _, _) => {
                match tuple {
                    ParsedTuple::Unit => vec![].into_boxed_slice(),
                    ParsedTuple::Single(s) => vec![s.as_ref()].into_boxed_slice(),
                    ParsedTuple::Multiple(m) => {
                        let mut v = vec![];
                        for e in m.iter() {
                            v.push(e);
                        }
                        v.into_boxed_slice()
                    }
                }
            }
            single => vec![single].into_boxed_slice()
        }
    }

}

//// Text

#[derive(PartialEq, Eq, Clone)]
pub struct ParsedText { pub str: Rc<str> }

impl Text<ParsedValue, ParsedText, ParsedDictionary, ParsedTable, ParsedCompound, ParsedTuple, ParsedTag> for ParsedText {

    fn as_str(&self) -> &str {
        &self.str
    }

}

//// Dictionary

/// A parsed dictionary.
#[derive(Clone)]
pub struct ParsedDictionary {
    pub entries: Vec<ParsedEntry>,
}

impl ParsedDictionary {

    pub fn empty() -> Self {
        ParsedDictionary { entries: vec![] }
    }

}

impl Dictionary<ParsedValue, ParsedText, ParsedDictionary, ParsedTable, ParsedCompound, ParsedTuple, ParsedTag> for ParsedDictionary {

    type EntryIterator<'b> = EntryIterator<'b>;

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn get(&self, index: usize) -> Option<Entry<ParsedValue>> {
        if let Some(entry) = self.entries.get(index) {
            Some(Entry(&entry.0, &entry.1))
        } else {
            None
        }
    }

    fn iter(&self) -> Self::EntryIterator<'_> {
        EntryIterator(self.entries.iter())
    }

}

#[derive(Clone)]
pub struct ParsedEntry(Rc<str>, ParsedValue);

pub struct EntryIterator<'a>(Iter<'a, ParsedEntry>);

impl <'a> Iterator for EntryIterator<'a> {

    type Item = Entry<'a, ParsedValue>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.0.next() {
            Some(Entry(&c.0, &c.1))
        } else {
            None
        }
    }

}

//// Tuple

/// A parsed tuple.
#[derive(Clone)]
pub enum ParsedTuple {
    Unit,
    Single(Box<ParsedValue>), // Value must be a ParsedTuple
    Multiple(Box<[ParsedValue]>),
}

impl Tuple<ParsedValue, ParsedText, ParsedDictionary, ParsedTable, ParsedCompound, Self, ParsedTag> for ParsedTuple {

    type TupleIterator<'b> = TupleIterator<'b>;

    fn len(&self) -> usize {
        match self {
            ParsedTuple::Unit => 0,
            ParsedTuple::Single(_) => 1,
            ParsedTuple::Multiple(m) => m.len(),
        }
    }

    fn is_empty(&self) -> bool {
        matches!(self, ParsedTuple::Unit)
    }

    fn get(&self, index: usize) -> Option<&ParsedValue> {
        match self {
            ParsedTuple::Unit => None,
            ParsedTuple::Single(v) => {
                Some(v.as_ref())
            }
            ParsedTuple::Multiple(m) => {
                m.get(index)
            }
        }
    }

    fn iter(&self) -> Self::TupleIterator<'_> {
        match self {
            ParsedTuple::Unit => TupleIterator::Unit,
            ParsedTuple::Single(v) => TupleIterator::Single(false, v),
            ParsedTuple::Multiple(m) => TupleIterator::Multiple(0, m.as_ref()),
        }
    }

}

pub enum TupleIterator<'a> {
    Unit,
    Single(bool, &'a ParsedValue),
    Multiple(usize, &'a [ParsedValue]),
}

impl <'a> Iterator for TupleIterator<'a> {

    type Item = &'a ParsedValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TupleIterator::Unit => None,
            TupleIterator::Single(done, v) => {
                if *done {
                    None
                } else {
                    *done = true;
                    Some(v)
                }
            }
            TupleIterator::Multiple(index, v) => {
                let v = v.get(*index);
                *index += 1;
                v
            }
        }
    }

}

//// Table

/// A parsed table.
#[derive(Clone)]
pub struct ParsedTable {
    pub elements: Vec<ParsedValue>,
    pub columns: usize,
}

impl ParsedTable {

    /// Empty table.
    pub fn empty() -> Self {
        ParsedTable { elements: vec![], columns: 0 }
    }

}

impl Table<ParsedValue, ParsedText, ParsedDictionary, Self, ParsedCompound, ParsedTuple, ParsedTag> for ParsedTable {

    type RowIterator<'b> = RowIterator<'b>;

    type EntryIterator<'b> = Iter<'b, ParsedValue>;

    fn len(&self) -> usize {
        self.elements.len()
    }

    fn columns(&self) -> usize {
       self.columns
    }

    fn rows(&self) -> usize {
        if self.columns != 0 {
            self.elements.len() / self.columns
        } else {
            0
        }
    }

    fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    fn is_list(&self) -> bool {
        self.columns <= 1
    }

    fn get_entry(&self, row: usize, column: usize) -> Option<&ParsedValue> {
        self.elements.get(row * self.columns + column)
    }

    fn get_row(&self, row: usize) -> Option<&[ParsedValue]> {
        if self.columns != 0 {
            Some(&self.elements[row * self.columns .. row * (self.columns + 1)])
        } else {
            None
        }
    }

    fn iter_entries(&self) -> Self::EntryIterator<'_> {
        self.elements.iter()
    }

    fn iter_rows(&self) -> Self::RowIterator<'_> {
        RowIterator { columns: self.columns, iter: self.elements.iter() }
    }

}

pub struct RowIterator<'a> {
    columns: usize,
    iter: Iter<'a, ParsedValue>,
}

impl <'a> Iterator for RowIterator<'a> {

    type Item = Box<[&'a ParsedValue]>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut row = vec![];
        let mut c = 0;
        while c < self.columns {
            if let Some(entry) = self.iter.next() {
                row.push(entry);
                c += 1;
            } else {
                return None;
            };
        };
        Some(row.into_boxed_slice())
    }

}

//// Compound

#[derive(Clone)]
pub struct ParsedCompound {
    components: Vec<ParsedValue>, // Todo reorganize
    whitespace: Vec<bool>,
}

impl ParsedCompound {

}

impl Compound<ParsedValue, ParsedText, ParsedDictionary, ParsedTable, Self, ParsedTuple, ParsedTag> for ParsedCompound {

    type ElementIterator<'b> = ElementIterator<'b>;

    fn len(&self) -> usize {
        self.components.len() + self.whitespace.len()
    }

    fn get(&self, index: usize) -> Option<Element<&ParsedValue>> {
        todo!()
    }

    fn iter(&self) -> Self::ElementIterator<'_> {
        ElementIterator {
            components: &self.components,
            whitespace: &self.whitespace,
            index: 0,
            after_component: false,
        }
    }

}

pub struct ElementIterator<'b> {
    components: &'b[ParsedValue],
    whitespace: &'b[bool],
    index: usize,
    after_component: bool,
}

impl <'b> Iterator for ElementIterator<'b> {

    type Item = Element<&'b ParsedValue>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index != self.components.len() - 1 {
            if self.after_component {
                let whitespace = self.whitespace[self.index];
                self.index += 1;
                if whitespace {
                    self.after_component = false;
                    return Some(Element::Space);
                };
            };
        } else {
            if self.after_component {
                return None;
            };
        };
        self.after_component = true;
        Some(Element::Solid(&self.components[self.index]))
    }

}

//// Tag

/// A parsed tag.
#[derive(Clone)]
pub struct ParsedTag {
    pub name: Rc<str>,
    pub attributes: Vec<ParsedAttribute>,
    pub value: Box<ParsedValue>,
}

impl Tag<ParsedValue, ParsedText, ParsedDictionary, ParsedTable, ParsedCompound, ParsedTuple, Self> for ParsedTag {

    type AttributeIterator<'b> = AttributeIterator<'b>;

    fn name(&self) -> &str {
        &self.name
    }

    fn has_attributes(&self) -> bool {
        !self.attributes.is_empty()
    }

    fn get_attribute_by(&self, key: &str) -> Option<AttributeValue<'_>> {
        for attribute in &self.attributes {
            if key.eq(attribute.key().deref()) {
                return match attribute {
                    ParsedAttribute(_, Some(v)) => Some(AttributeValue(Some(&v))),
                    ParsedAttribute(_, None) => Some(AttributeValue(None)),
                };
            }
        }
        None
    }

    fn get_attribute_at(&self, index: usize) -> Option<Attribute<'_>> {
        if let Some(attribute) = self.attributes.get(index) {
            match attribute {
                ParsedAttribute(k, Some(v)) => Some(Attribute(&k, Some(&v))),
                ParsedAttribute(k, None) => Some(Attribute(&k, None)),
            }
        } else {
            None
        }
    }

    fn iter_attributes(&self) -> Self::AttributeIterator<'_> {
        AttributeIterator { iter: self.attributes.iter() }
    }

    fn get(&self) -> &ParsedValue {
        &self.value
    }

}

#[derive(Clone)]
pub struct ParsedAttribute(Rc<str>, Option<Rc<str>>);

impl ParsedAttribute {

    fn key(&self) -> Rc<str> {
        self.0.clone()
    }

}

pub struct AttributeIterator<'a> {
    iter: Iter<'a, ParsedAttribute>,
}

impl <'a> Iterator for AttributeIterator<'a> {

    type Item = Attribute<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ParsedAttribute(key, value)) = self.iter.next() {
            if let Some(value) = value {
                Some(Attribute(key, Some(value)))
            } else {
                Some(Attribute(key, None))
            }
        } else {
            None
        }
    }

}

//// Parsing result formatting

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TokenType {
    Whitespace, Word, Transcription, TextBlock,
    Colon, Semicolon, Bar,
    Ampersand, Tilde, Diamond,
    LeftBracket, RightBracket, LeftSquare, RightSquare, LeftAngle, RightAngle,
    End,
}

impl Display for TokenType {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Whitespace => write!(f, "Whitespace"),
            TokenType::Word => write!(f, "Word"),
            TokenType::Transcription => write!(f, "Transcription"),
            TokenType::TextBlock => write!(f, "TextBlock"),
            TokenType::Colon => write!(f, ":"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Bar => write!(f, "|"),
            TokenType::Ampersand => write!(f, "&"),
            TokenType::Tilde => write!(f, "~"),
            TokenType::Diamond => write!(f, "<>"),
            TokenType::LeftBracket => write!(f, "{{"),
            TokenType::RightBracket => write!(f, "}}"),
            TokenType::LeftSquare => write!(f, "["),
            TokenType::RightSquare => write!(f, "]"),
            TokenType::LeftAngle => write!(f, "<"),
            TokenType::RightAngle => write!(f, ">"),
            TokenType::End => write!(f, "End"),
        }
    }

}

impl Token {

    fn to_type(&self) -> TokenType {
        match self {
            Token::Whitespace(..) => TokenType::Whitespace,
            Token::Word(..) => TokenType::Word,
            Token::Transcription(..) => TokenType::Transcription,
            Token::TextBlock(..) => TokenType::TextBlock,
            Token::Colon(..) => TokenType::Colon,
            Token::Semicolon(..) => TokenType::Semicolon,
            Token::Bar(..) => TokenType::Bar,
            Token::Ampersand(..) => TokenType::Ampersand,
            Token::Tilde(..) => TokenType::Tilde,
            Token::Diamond(..) => TokenType::Diamond,
            Token::LeftBracket(..) => TokenType::LeftBracket,
            Token::RightBracket(..) => TokenType::RightBracket,
            Token::LeftSquare(..) => TokenType::LeftSquare,
            Token::RightSquare(..) => TokenType::RightSquare,
            Token::LeftAngle(..) => TokenType::LeftAngle,
            Token::RightAngle(..) => TokenType::RightAngle,
            Token::End(..) => TokenType::End,
        }
    }

}
