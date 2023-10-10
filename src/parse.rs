//! Parsing of Khi documents.
//!
//! The root element of a document can be an expression, a sequence or a dictionary.
//! Use the corresponding function to parse a document: either [parse_expression_document],
//! [parse_sequence_document] or [parse_dictionary_document].

use std::borrow::Borrow;
use std::collections::{HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::rc::Rc;
use std::slice::Iter;
use ref_cast::RefCast;
use crate::{Dictionary, Directive, Expression, Table, Text, Component, WhitespaceOption};
use crate::lex::{lex, LexError, Position, Token};

//// Parse

/// Parse a document with an expression root node.
pub fn parse_expression_document(document: &str) -> Result<ParsedExpression> {
    let tokens = tokenize(document)?;
    let mut iter = TokenIter::new(tokens.iter());
    let mut strings = HashSet::new();
    let structure = parse_nullable_expression(&mut iter, &mut strings)?;
    if !matches!(iter.t, Token::End(..)) {
        return Err(ParseError::ExpectedEnd(iter.position()));
    };
    Ok(structure)
}

/// Parse a document with a table root node.
pub fn parse_table_document(document: &str) -> Result<ParsedTable> {
    let tokens = tokenize(document)?;
    let mut iter = TokenIter::new(tokens.iter());
    let mut strings = HashSet::new();
    let sequence = parse_nullable_table(&mut iter, &mut strings)?;
    if !matches!(iter.t, Token::End(..)) {
        return Err(ParseError::ExpectedEnd(iter.position()));
    };
    Ok(sequence)
}

/// Parse a document with a dictionary root node.
pub fn parse_dictionary_document(document: &str) -> Result<ParsedDictionary> {
    let tokens = tokenize(document)?;
    let mut iter = TokenIter::new(tokens.iter());
    let mut strings = HashSet::new();
    let dictionary = parse_nullable_dictionary(&mut iter, &mut strings)?;
    if !matches!(iter.t, Token::End(..) ) {
        return Err(ParseError::ExpectedEnd(iter.position()));
    };
    Ok(dictionary)
}

/// Convert a Khi document to tokens.
fn tokenize(document: &str) -> Result<Vec<Token>> {
    let chars = document.chars();
    let tokens = match lex(chars) {
        Ok(ok) => ok,
        Err(err) => {
            return match err {
                LexError::EscapeEOS => Err(ParseError::EscapingEndOfStream),
                LexError::CommentedBracket(at) => Err(ParseError::CommentedBracket(at)),
                LexError::UnclosedQuote(at) => Err(ParseError::UnclosedQuote(at)),
                LexError::UnknownEscapeSequence(at) => Err(ParseError::UnknownEscapeSequence(at)),
                LexError::InvalidHashSequence(at) => Err(ParseError::UnknownHashSequence(at)),
            };
        }
    };
    Ok(tokens)
}

pub type Result<T> = std::result::Result<T, ParseError>;

//// Token iterator

/// Token iterator.
struct TokenIter<'a> {
    tokens: Iter<'a, Token>,
    t: &'a Token,
    t2: &'a Token,
}

impl <'a> TokenIter<'a> {

    fn new(mut tokens: Iter<'a, Token>) -> Self {
        let t = tokens.next().unwrap();
        let t2 = tokens.next().unwrap_or(t);
        TokenIter { tokens, t, t2 }
    }

    fn next(&mut self) {
        self.t = self.t2;
        self.t2 = self.tokens.next().unwrap_or(self.t);
    }

    fn skip_whitespace(&mut self) -> bool {
        let mut skipped = false;
        loop {
            if matches!(self.t, Token::Whitespace(..)) {
                skipped = true;
                self.next();
            } else {
                break;
            };
        };
        skipped
    }

    fn position(&self) -> Position {
        self.t.position()
    }

    fn peek_next_glyph_token(&mut self) -> &Token {
        match self.t {
            Token::Whitespace(..) => {
                self.skip_lookahead_whitespace();
                self.t2
            }
            t => t,
        }
    }

    fn skip_lookahead_whitespace(&mut self) {
        loop {
            if matches!(self.t2, Token::Whitespace(..)) {
                self.t2 = self.tokens.next().unwrap_or(self.t2);
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
        self.t.to_type()
    }

    fn expectation_error<T>(&self, token_type: &'static[TokenType]) -> Result<T> {
        Err(ParseError::Expected(token_type, self.token_type(), self.position()))
    }

}

//// Parser
////
//// Approximate recursive descent, but with modifications that require no backtracking
//// or large lookahead.

/// Parse a nullable expression.
///
/// Recognizes `<null-expr>`.
///
/// ```text
/// # Nullable expression
/// # Spans Expression object.
/// <null-expr> = <>
///             | <expr>
/// ```
fn parse_nullable_expression<'a>(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedExpression> {
    if matches!(iter.peek_next_glyph_token(), Token::Word(..) | Token::Quote(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..)) {
        parse_expression(iter, strings)
    } else {
        let from = iter.position();
        iter.skip_whitespace();
        let to = iter.position();
        Ok(ParsedComponent::empty(from, to).into())
    }
}

/// Parse an expression.
///
/// Recognizes `<expr>`.
///
/// ```text
/// # Expression
/// <expr> = <><sentence><>
///        | <><sentence> <expr-tail-no-text><>
///        | <><expr-tail-no-text><>
/// ```
fn parse_expression(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedExpression> {
    let components: Vec<ParsedComponent> = vec![];
    let whitespace = vec![];
    let from = iter.position();
    let after_whitespace = iter.skip_whitespace();
    if !matches!(iter.t, Token::Word(..) | Token::Quote(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..)) {
        return iter.expectation_error(&[TokenType::Word, TokenType::Quote, TokenType::LeftBracket, TokenType::LeftSquare, TokenType::LeftAngle]);
    };
    parse_expression_tail_with(iter, strings, from, components, whitespace, after_whitespace)
}

/// Parse a nullable expression tail with an initial state.
///
/// Recognizes `<>` and `<><expr-tail>`.
fn parse_nullable_expression_tail_with(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>, from: Position, components: Vec<ParsedComponent>, whitespace: Vec<bool>) -> Result<ParsedExpression> {
    let components = components;
    let whitespace = whitespace;
    let after_whitespace = iter.skip_whitespace();
    if matches!(iter.t, Token::Word(..) | Token::Quote(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..)) {
        parse_expression_tail_with(iter, strings, from, components, whitespace, after_whitespace)
    } else {
        iter.skip_whitespace();
        let to = iter.position();
        Ok(ParsedComponent::from_components(from, to, components, whitespace).into())
    }
}

/// Parse an expression tail with an initial state.
///
/// Recognizes `<><expr-tail><>`.
///
/// ```text
/// # Component of an expression
/// <expr-tail> = <sentence> <expr-tail-no-text>
///             | <expr-tail-no-text>
///
/// <expr-tail-no-text> = <quote>
///                     | <quote> <expr-tail>
///                     | <expr-cmp>
///                     | <expr-cmp> <expr-tail>
///                     | <dict-cmp>
///                     | <dict-cmp> <expr-tail>
///                     | <table-cmp>
///                     | <table-cmp> <expr-tail>
///                     | <closed-cmd-expr>
///                     | <closed-cmd-expr> <expr-tail>
///                     | <open-cmd-expr>
///                     | <open-cmd-expr>_<expr-tail>
///                     | <open-cmd-expr><expr-tail-no-text>
/// ```
fn parse_expression_tail_with(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>, from: Position, components: Vec<ParsedComponent>, whitespace: Vec<bool>, after_whitespace: bool) -> Result<ParsedExpression> {
    let mut components = components;
    let mut whitespace = whitespace;
    let mut after_whitespace = after_whitespace;
    loop {
        match iter.t {
            Token::Word(..) => push_component(&mut components, &mut whitespace, &mut after_whitespace, parse_sentence(iter, strings)?.into()),
            Token::Quote(..) => push_component(&mut components, &mut whitespace, &mut after_whitespace, parse_quote(iter, strings)?.into()),
            Token::LeftBracket(..) => push_component(&mut components, &mut whitespace, &mut after_whitespace, parse_bracket_component(iter, strings)?),
            Token::LeftSquare(..) => push_component(&mut components, &mut whitespace, &mut after_whitespace, parse_table_component(iter, strings)?.into()),
            Token::LeftAngle(..) => push_component(&mut components, &mut whitespace, &mut after_whitespace, parse_command_expression(iter, strings)?.into()),
            Token::Whitespace(..) => {
                iter.skip_whitespace();
                after_whitespace = true;
            },
            _ => break,
        };
    };
    let to = iter.position();
    return Ok(ParsedComponent::from_components(from, to, components, whitespace).into());
    fn push_component(components: &mut Vec<ParsedComponent>, whitespace: &mut Vec<bool>, after_whitespace: &mut bool, component: ParsedComponent) {
        if components.len() != 0 {
            if *after_whitespace {
                whitespace.push(true);
            } else {
                whitespace.push(false);
            };
            *after_whitespace = false;
        };
        components.push(component);
    }
}

/// Parse a bracket component.
///
/// Recognizes `<expr-cmp>` and `<dict-cmp>`.
///
/// ```text
/// # Expression component
/// # Spans an ExpressionComponent object.
/// <expr-cmp> = "{"<expr>"}"
///
/// # Dictionary component
/// # Spans a DictionaryComponent object.
/// <dict-cmp> = "{"<dict-head>"}"
///            | "{"<dict>"#}"
/// ```
fn parse_bracket_component(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedComponent> {
    if !matches!(iter.t, Token::LeftBracket(..)) {
        return iter.expectation_error(&[TokenType::LeftBracket]);
    };
    iter.next();
    let from = iter.position();
    iter.skip_whitespace();
    return match iter.t {
        Token::Word(..) => {
            let (candidate, candidate_from, candidate_to) = parse_word(iter, strings)?;
            match iter.peek_next_glyph_token() {
                Token::Word(..) => handle_sentence_after_plain_key(iter, strings, candidate_from, candidate, from),
                Token::Quote(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..) => handle_component_after_key(iter, strings, from, candidate, candidate_from, candidate_to),
                Token::Colon(..) => handle_colon_after_key(iter, strings, candidate, from),
                Token::Semicolon(..) => handle_semicolon_after_key(iter, strings, candidate, from),
                Token::Whitespace(..) => unreachable!(),
                _ => handle_closing_after_key(iter, strings, candidate, candidate_from, candidate_to),
            }
        }
        Token::Quote(.., quote) => {
            let quote = String::from(quote);
            let quote_from = iter.position();
            iter.next();
            let quote_to = iter.position();
            match iter.peek_next_glyph_token() {
                Token::Word(..) | Token::Quote(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..) => handle_component_after_key(iter, strings, from, quote, quote_from, quote_to),
                Token::Colon(..) => handle_colon_after_key(iter, strings, quote, from),
                Token::Semicolon(..) => handle_semicolon_after_key(iter, strings, quote, from),
                Token::Whitespace(..) => unreachable!(),
                _ => handle_closing_after_key(iter, strings, quote, quote_from, quote_to),
            }
        }
        Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..) => {
            let expression = parse_expression(iter, strings)?;
            if !matches!(iter.t, Token::RightBracket(..)) {
                return iter.expectation_error(&[TokenType::RightBracket]);
            };
            iter.next();
            Ok(expression.into())
        }
        Token::RightBracket(..) => {
            let to = iter.position();
            iter.next();
            Ok(ParsedComponent::Empty(from, to))
        }
        Token::HashRightBracket(..) => {
            let to = iter.position();
            iter.next();
            Ok(ParsedDictionary { entries: vec![], from, to }.into())
        }
        Token::Whitespace(..) => unreachable!(),
        _ => return iter.expectation_error(&[TokenType::Word, TokenType::Quote, TokenType::LeftBracket, TokenType::LeftSquare, TokenType::LeftAngle, TokenType::RightBracket, TokenType::HashRightBracket]),
    };
    fn handle_colon_after_key(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>, candidate: String, from: Position) -> Result<ParsedComponent> {
        iter.consume_next_glyph_token();
        let key = store_str(strings, &candidate);
        let value = parse_expression(iter, strings)?;
        let entry = (key, value);
        let entries = vec![entry];
        if matches!(iter.t, Token::Semicolon(..)) {
            iter.next();
            if matches!(iter.peek_next_glyph_token(), Token::Word(..) | Token::Quote(..)) {
                let dictionary = parse_dictionary_with(iter, strings, from, entries)?.into();
                if !matches!(iter.t, Token::RightBracket(..) | Token::HashRightBracket(..)) {
                    return iter.expectation_error(&[TokenType::RightBracket, TokenType::HashRightBracket]);
                };
                iter.next();
                Ok(dictionary)
            } else {
                iter.skip_whitespace();
                let to = iter.position();
                if !matches!(iter.t, Token::RightBracket(..) | Token::HashRightBracket(..)) {
                    return iter.expectation_error(&[TokenType::RightBracket, TokenType::HashRightBracket]);
                };
                iter.next();
                Ok(ParsedDictionary { entries, from, to }.into())
            }
        } else {
            let to = iter.position();
            if !matches!(iter.t, Token::RightBracket(..) | Token::HashRightBracket(..)) {
                return iter.expectation_error(&[TokenType::RightBracket, TokenType::HashRightBracket]);
            };
            iter.next();
            Ok(ParsedDictionary { entries, from, to }.into())
        }
    }
    fn handle_semicolon_after_key(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>, candidate: String, from: Position) -> Result<ParsedComponent> {
        iter.skip_whitespace();
        let at = iter.position();
        iter.next();
        let key = store_str(strings, &candidate);
        let value = ParsedComponent::Empty(at, at).into();
        let entry = (key, value);
        let entries = vec![entry];
        if matches!(iter.peek_next_glyph_token(), Token::Word(..) | Token::Quote(..)) {
            let dictionary = parse_dictionary_with(iter, strings, from, entries)?.into();
            if !matches!(iter.t, Token::RightBracket(..) | Token::HashRightBracket(..)) {
                return iter.expectation_error(&[TokenType::RightBracket, TokenType::HashRightBracket]);
            };
            iter.next();
            Ok(dictionary)
        } else {
            iter.skip_whitespace();
            let to = iter.position();
            if !matches!(iter.t, Token::RightBracket(..) | Token::HashRightBracket(..)) {
                return iter.expectation_error(&[TokenType::RightBracket, TokenType::HashRightBracket]);
            };
            iter.next();
            Ok(ParsedDictionary { entries, from, to }.into())
        }
    }
    fn handle_component_after_key(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>, from: Position, candidate: String, candidate_from: Position, candidate_to: Position) -> Result<ParsedComponent> {
        let candidate = store_str(strings, &candidate);
        let text = ParsedText { str: candidate, from: candidate_from, to: candidate_to }.into();
        let components = vec![text];
        let after_whitespace = iter.skip_whitespace();
        let expression = parse_expression_tail_with(iter, strings, from, components, vec![], after_whitespace)?.into();
        if !matches!(iter.t, Token::RightBracket(..)) {
            return iter.expectation_error(&[TokenType::RightBracket]);
        };
        iter.next();
        Ok(expression)
    }
    fn handle_sentence_after_plain_key(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>, text_from: Position, mut word: String, from: Position) -> Result<ParsedComponent> {
        iter.skip_whitespace();
        word.push(' ');
        let sentence = parse_sentence_with(iter, strings, text_from, word)?.into();
        let components = vec![sentence];
        let expression = parse_nullable_expression_tail_with(iter, strings, from, components, vec![])?.into();
        if !matches!(iter.t, Token::RightBracket(..)) {
            return iter.expectation_error(&[TokenType::RightBracket]);
        };
        iter.next();
        Ok(expression)
    }
    fn handle_closing_after_key(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>, key: String, key_from: Position, key_to: Position) -> Result<ParsedComponent> {
        iter.skip_whitespace();
        let str = store_str(strings, &key);
        if !matches!(iter.t, Token::RightBracket(..)) {
            return iter.expectation_error(&[TokenType::RightBracket]);
        };
        iter.next();
        Ok(ParsedText { str, from: key_from, to: key_to }.into())
    }
}

/// Parse a nullable dictionary.
///
/// Recognizes `<null-dict>`.
///
/// ```text
/// # Nullable dictionary
/// <null-dict> = <>
///             | <dict>
/// ```
fn parse_nullable_dictionary(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedDictionary> {
    match iter.peek_next_glyph_token() {
        Token::Word(..) | Token::Quote(..) => parse_dictionary(iter, strings),
        Token::Whitespace(..) => unreachable!(),
        _ => {
            let from = iter.position();
            iter.skip_whitespace();
            let to = iter.position();
            Ok(ParsedDictionary::empty(from, to))
        }
    }
}

/// Parse a dictionary.
///
/// Recognizes `<dict>`.
///
/// ```text
/// # Dictionary
/// <dict> = <><word> ":"<expr>               # Last entry
///        | <><word> ":"<expr>";"<>          # Last entry with trailing semicolon
///        | <><word> ":"<expr>";"<dict>      # Intermediary entry
///        | <><word> ";"<>                   # Last key
///        | <><word> ";"<dict>               # Intermediary key
/// ```
fn parse_dictionary(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedDictionary> {
    let entries = vec![];
    let from = iter.position();
    parse_dictionary_with(iter, strings, from, entries)
}

/// Parse a dictionary with an initial state.
///
/// Recognizes `<dict>`.
///
/// ```text
/// # Dictionary
/// <dict> = <><word> ":"<expr>          # Last entry
///        | <><word> ":"<expr>";"<>     # Last entry with trailing semicolon
///        | <><word> ":"<expr>";"<dict> # Intermediary entry
///        | <><word> ";"<>              # Last key
///        | <><word> ";"<dict>          # Intermediary key
/// ```
fn parse_dictionary_with(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>, from: Position, with: Vec<(Rc<str>, ParsedExpression)>) -> Result<ParsedDictionary> {
    let mut entries = with;
    if !matches!(iter.peek_next_glyph_token(), Token::Word(..) | Token::Quote(..)) {
        iter.skip_whitespace();
        return iter.expectation_error(&[TokenType::Word, TokenType::Quote]);
    };
    loop {
        iter.skip_whitespace();
        let (key, _, _) = parse_word(iter, strings)?;
        let key = store_str(strings, &key);
        iter.skip_whitespace();
        match iter.t {
            Token::Colon(..) => {
                iter.next();
                let value = parse_expression(iter, strings)?;
                let entry = (key, value);
                entries.push(entry);
                if !matches!(iter.t, Token::Semicolon(..)) {
                    let to = iter.position();
                    return Ok(ParsedDictionary { entries, from, to });
                };
                iter.next();
                if !matches!(iter.peek_next_glyph_token(), Token::Word(..) | Token::Quote(..)) {
                    iter.skip_whitespace();
                    let to = iter.position();
                    return Ok(ParsedDictionary { entries, from, to });
                };
            }
            Token::Semicolon(..) => {
                let at = iter.position();
                let value = ParsedComponent::Empty(at, at).into();
                let entry = (key, value);
                entries.push(entry);
                iter.next();
                if !matches!(iter.peek_next_glyph_token(), Token::Word(..) | Token::Quote(..)) {
                    iter.skip_whitespace();
                    let to = iter.position();
                    return Ok(ParsedDictionary { entries, from, to });
                };
            }
            Token::Whitespace(..) => unreachable!(),
            _ => return iter.expectation_error(&[TokenType::Colon, TokenType::Semicolon]),
        };
    };
}

/// Parse a nullable table.
///
/// Recognizes `<null-table>`.
///
/// ```text
/// <null-table> = <>
///              | <table>
/// ```
fn parse_nullable_table(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedTable> {
    match iter.peek_next_glyph_token() {
        Token::Word(..) | Token::Quote(..) | Token::Bar(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..) | Token::Colon(..) => {
            parse_table(iter, strings)
        }
        Token::Whitespace(_) => unreachable!(),
        _ => {
            let from = iter.position();
            iter.skip_whitespace();
            let to = iter.position();
            Ok(ParsedTable { elements: vec![], from, to, columns: 0 })
        },
    }
}

/// Parse a table.
///
/// Recognizes `<table>`.
///
/// ```text
/// # Table
/// <table> = <seq>
///         | <><tab><>
///
/// # Table in sequential notation
/// # Though not encoded in the grammar, all rows must have the same number of columns.
/// <seq> = <expr>         # Element, end
///       | <expr>";"<>    # Element with trailing semicolon
///       | <expr>";"<seq> # Element, next row
///       | <expr>"|"<seq> # Element, next column
///
/// # Table in tabular notation
/// # Though not encoded in the grammar, all rows must have the same number of columns.
/// <tab> = "|"<expr><tab> # Append entry to row
///       | "|" <tab>      # Next row
///       | "|"            # End
/// ```
fn parse_table(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedTable> {
    match iter.peek_next_glyph_token() {
        Token::Word(..) | Token::Quote(..) | Token::LeftSquare(..) | Token::LeftBracket(..) | Token::LeftAngle(..) | Token::Colon(..) => parse_sequence_notation(iter, strings),
        Token::Bar(..) => {
            let from = iter.position();
            iter.skip_whitespace();
            let (elements, columns) = parse_tabular_notation(iter, strings)?;
            iter.skip_whitespace();
            let to = iter.position();
            Ok(ParsedTable { elements, from, to, columns })
        },
        Token::Whitespace(..) => unreachable!(),
        _ => {
            iter.skip_whitespace();
            iter.expectation_error(&[TokenType::Word, TokenType::Quote, TokenType::LeftSquare, TokenType::LeftBracket, TokenType::LeftAngle, TokenType::Bar])
        }
    }
}

fn parse_empty_expression_colon(iter: &mut TokenIter) -> ParsedExpression {
    iter.skip_whitespace();
    let colon_from = iter.position();
    iter.next();
    let colon_to = iter.position();
    iter.skip_whitespace();
    ParsedComponent::Empty(colon_from, colon_to).into_expression()
}

/// Parse a table written in sequence notation.
///
/// Recognizes `<seq>`.
///
/// ```text
/// # Table in sequential notation
/// <seq> = <expr>         # Last element
///       | <>":"<>        # Empty element
///       | <expr>";"<>    # Last element with trailing semicolon
///       | <>":" ";"<>    # Empty last element with trailing semicolon
///       | <expr>";"<seq> # Last element of row
///       | <>":" ";"<seq> # Empty last element of row
///       | <expr>"|"<seq> # Intermediary element
///       | <>":" "|"<seq> # Empty intermediary element
/// ```
fn parse_sequence_notation(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedTable> {
    let mut elements = vec![];
    let mut columns = 0;
    let from = iter.position();
    loop { // Read the first row, and determine number of columns.
        let element = if !matches!(iter.peek_next_glyph_token(), Token::Colon(..)) {
            parse_expression(iter, strings)?
        } else {
            parse_empty_expression_colon(iter)
        };
        elements.push(element);
        columns += 1;
        match iter.t {
            Token::Semicolon(..) => {
                iter.next();
                if matches!(iter.peek_next_glyph_token(), Token::Word(..) | Token::Quote(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..) | Token::Colon(..)) {
                    break;
                } else {
                    iter.skip_whitespace();
                    let to = iter.position();
                    return Ok(ParsedTable { elements, from, to, columns });
                };
            }
            Token::Bar(..) => {
                iter.next();
                // if !matches!(iter.peek_next_glyph_token(), Token::Word(..) | Token::Quote(..) | Token::Query(..) | Token::HashQuery(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..) | Token::LeftAnglePlus(..) | Token::Colon(..)) {
                //     iter.skip_whitespace();
                //     return iter.expectation_error(&[TokenType::Word, TokenType::Quote, TokenType::Query, TokenType::HashQuery, TokenType::LeftBracket, TokenType::LeftSquare, TokenType::LeftAngle, TokenType::LeftAnglePlus, TokenType::Colon])
                // };
            },
            Token::Colon(..) | Token::Diamond(..) | Token::RightSquare(..) | Token::HashRightSquare(..) | Token::RightBracket(..) | Token::HashRightBracket(..) | Token::RightAngle(..) | Token::End(..) => {
                let to = iter.position();
                return Ok(ParsedTable { elements, from, to, columns });
            },
            Token::Word(..) | Token::Quote(..) | Token::LeftSquare(..) | Token::LeftBracket(..) | Token::LeftAngle(..) | Token::Whitespace(..) => unreachable!(),
        }
    };
    loop {
        let mut c = 0;
        loop {
            let element = if !matches!(iter.peek_next_glyph_token(), Token::Colon(..)) {
                parse_expression(iter, strings)?
            } else {
                parse_empty_expression_colon(iter)
            };
            elements.push(element);
            c += 1;
            match iter.t {
                Token::Semicolon(..) => {
                    if c != columns {
                        return Err(ParseError::ExpectedColumns(iter.position(), c, columns));
                    };
                    iter.next();
                    if matches!(iter.peek_next_glyph_token(), Token::Word(..) | Token::Quote(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..) | Token::Colon(..)) {
                        break;
                    } else {
                        iter.skip_whitespace();
                        let to = iter.position();
                        return Ok(ParsedTable { elements, from, to, columns });
                    };
                }
                Token::Bar(..) => iter.next(),
                Token::Colon(..) | Token::Diamond(..) | Token::RightSquare(..) | Token::HashRightSquare(..) | Token::RightBracket(..) | Token::HashRightBracket(..) | Token::RightAngle(..) | Token::End(..) => {
                    if c != columns {
                        return Err(ParseError::ExpectedColumns(iter.position(), c, columns));
                    };
                    let to = iter.position();
                    return Ok(ParsedTable { elements, from, to, columns });
                },
                Token::Word(..) | Token::Quote(..) | Token::LeftSquare(..) | Token::LeftBracket(..) | Token::LeftAngle(..) | Token::Whitespace(..) => unreachable!(),
            }
        };
    }
}

/// Parse a table written in tabular notation.
///
/// Recognizes `<tab>`.
///
/// ```text
/// # Table in tabular notation
/// <tab> = "|"<expr><tab> # Intermediary element
///       | "|" ":" <tab>  # Empty element
///       | "|" <tab>      # End of row
///       | "|"            # End
/// ```
fn parse_tabular_notation(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<(Vec<ParsedExpression>, usize)> {
    let mut elements = vec![];
    let mut columns = 0;
    loop { // Parse the first row.
        if !matches!(iter.t, Token::Bar(..)) {
            return iter.expectation_error(&[TokenType::Bar]);
        };
        iter.next();
        match iter.peek_next_glyph_token() {
            Token::Word(..) | Token::Quote(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..) => {
                let expression = parse_expression(iter, strings)?;
                columns += 1;
                elements.push(expression);
            }
            Token::Colon(..) => {
                let expression = parse_empty_expression_colon(iter);
                columns += 1;
                elements.push(expression);
            }
            Token::Bar(..) => {
                iter.skip_whitespace();
                if columns == 0 {
                    return Err(ParseError::ZeroColumns(iter.position()));
                };
                break;
            }
            Token::Whitespace(..) => unreachable!(),
            _ => {
                if columns == 0 {
                    return Err(ParseError::ZeroColumns(iter.position()));
                };
                return Ok((elements, columns));
            }
        }
    };
    loop { // Parse the remaining rows.
        let mut c = 0;
        loop {
            if !matches!(iter.t, Token::Bar(..)) {
                return iter.expectation_error(&[TokenType::Bar]);
            };
            iter.next();
            match iter.peek_next_glyph_token() {
                Token::Word(..) | Token::Quote(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..) => {
                    let expression = parse_expression(iter, strings)?;
                    c += 1;
                    elements.push(expression);
                }
                Token::Colon(..) => {
                    let expression = parse_empty_expression_colon(iter);
                    c += 1;
                    elements.push(expression);
                }
                Token::Bar(..) => {
                    iter.skip_whitespace();
                    if c != columns {
                        return Err(ParseError::ExpectedColumns(iter.position(), c, columns))
                    };
                    break;
                }
                Token::Whitespace(..) => unreachable!(),
                _ => {
                    if c != columns {
                        return Err(ParseError::ExpectedColumns(iter.position(), c, columns))
                    };
                    return Ok((elements, columns));
                }
            };
        };
    };
}

/// Parse a table component.
///
/// Recognizes `<table-cmp>`.
///
/// ```text
/// # Table component
/// <table-cmp> = "["<table>"]"
///             | "["<null-table>"#]"
/// ```
fn parse_table_component(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedTable> {
    if !matches!(iter.t, Token::LeftSquare(..)) {
        return iter.expectation_error(&[TokenType::LeftSquare]);
    };
    iter.next();
    match iter.peek_next_glyph_token() {
        Token::Word(..) | Token::Quote(..) | Token::Bar(..) | Token::LeftBracket(..) | Token::LeftSquare(..) | Token::LeftAngle(..) => {
            let table = parse_table(iter, strings)?;
            if !matches!(iter.t, Token::RightSquare(..) | Token::HashRightSquare(..)) {
                return iter.expectation_error(&[TokenType::RightSquare, TokenType::HashRightSquare]);
            };
            iter.next();
            Ok(table)
        }
        Token::HashRightSquare(..) => {
            let from = iter.position();
            iter.skip_whitespace();
            let to = iter.position();
            Ok(ParsedTable { elements: vec![], from, to, columns: 0 })
        }
        Token::Whitespace(..) => unreachable!(),
        _ => {
            iter.skip_whitespace();
            iter.expectation_error(&[TokenType::Word, TokenType::Quote, TokenType::Bar, TokenType::LeftBracket, TokenType::LeftSquare, TokenType::LeftAngle, TokenType::HashRightSquare])
        },
    }
}

/// Parse a command.
///
/// ```text
/// # Command
/// <cmd> = "<"<word> ">"
///       | "<"<word> <attr> ">"
/// ```
fn parse_command(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<(String, Vec<(Rc<str>, ParsedExpression)>)> {
    if !matches!(iter.t, Token::LeftAngle(..)) {
        return iter.expectation_error(&[TokenType::LeftAngle]);
    };
    iter.next();
    parse_directive_tail(iter, strings)
}

/// Parse the remainder of a command.
fn parse_directive_tail(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<(String, Vec<(Rc<str>, ParsedExpression)>)> {
    if !matches!(iter.t, Token::Word(..)) {
        return iter.expectation_error(&[TokenType::Word]);
    };
    let (directive, _, _) = parse_word(iter, strings)?;
    iter.skip_whitespace();
    let attributes = if matches!(iter.t, Token::Word(..)) {
        parse_attributes(iter, strings)?
    } else {
        vec![]
    };
    iter.skip_whitespace();
    if !matches!(iter.t, Token::RightAngle(..)) {
        return iter.expectation_error(&[TokenType::RightAngle]);
    };
    iter.next();
    Ok((directive, attributes))
}

/// Parse attributes.
///
/// Recognizes `<attr>.`
///
/// ```text
/// # Attribute
/// <attr> = <word>
///        | <word>_<attr>
///        | <word>":"<word>
///        | <word>":"<word>_<attr>
///        | <word>":"<quote>
///        | <word>":"<quote> <attr>
///        | <word>":"<expr-cmp>
///        | <word>":"<expr-cmp> <attr>
///        | <word>":"<dict-cmp>
///        | <word>":"<dict-cmp> <attr>
///        | <word>":"<table-cmp>
///        | <word>":"<table-cmp> <attr>
/// ```
fn parse_attributes(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<Vec<(Rc<str>, ParsedExpression)>> {
    let mut attributes = vec![];
    loop {
        let (key, _key_from, _key_to) = parse_word(iter, strings)?;
        let value = match iter.t {
            Token::Colon(..) => {
                iter.next();
                match iter.t {
                    Token::Word(..) => parse_text_word(iter, strings)?.into(),
                    Token::Quote(..) => parse_quote(iter, strings)?.into(),
                    Token::LeftBracket(..) => parse_bracket_component(iter, strings)?.into(),
                    Token::LeftSquare(..) => parse_table_component(iter, strings)?.into(),
                    _ => return iter.expectation_error(&[TokenType::Word, TokenType::Quote, TokenType::LeftBracket, TokenType::LeftSquare])
                }
            }
            Token::Word(..) => unreachable!(),
            _ => {
                let at = iter.position();
                ParsedComponent::Empty(at, at).into()
            }
        };
        let key = store_str(strings, &key);
        let attribute = (key, value);
        attributes.push(attribute);
        if !matches!(iter.peek_next_glyph_token(), Token::Word(..)) {
            return Ok(attributes);
        };
        iter.skip_whitespace();
    };
}

/// Parse a command argument.
///
/// Recognizes `<cmd>`.
fn parse_command_argument(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedDirective> {
    let from = iter.position();
    let (directive, attributes) = parse_command(iter, strings)?;
    let directive = store_str(strings, &directive);
    let to = iter.position();
    Ok(ParsedDirective { directive, attributes, arguments: vec![], from, to })
}

/// Parse a command expression.
///
/// Recognizes `<open-cmd-expr>` and `<closed-cmd-expr>`.
///
/// ```text
/// # Open command expression
/// <open-cmd-expr> = <cmd><open-cmd-arg>
///
/// # Command argument
/// <open-cmd-arg> = ":"<arg-word>
///                | ":"<arg-word><cmd-arg>
///                | ":"<quote><cmd-arg>
///                | ":"<expr-cmp><cmd-arg>
///                | ":"<table-cmp><cmd-arg>
///                | ":"<dict-cmp><cmd-arg>
///                | ":"<cmd><cmd-arg>
///                | ":""<>"":"<open-cmd-expr>
///
/// <closed-cmd-expr> = <cmd><closed-cmd-arg>
///
/// <closed-cmd-arg> = ":"<arg-word><cmd-arg>
///                  | ":"<quote><cmd-arg>
///                  | ":"<quote>
///                  | ":"<expr-cmp><cmd-arg>
///                  | ":"<expr-cmp>
///                  | ":"<table-cmp><cmd-arg>
///                  | ":"<table-cmp>
///                  | ":"<dict-cmp><cmd-arg>
///                  | ":"<dict-cmp>
///                  | ":"<cmd><cmd-arg>
///                  | ":"<cmd>
///                  | ":""<>"":"<closed-cmd-expr>
/// ```
fn parse_command_expression(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedDirective> {
    let from = iter.position();
    let (directive, attributes) = parse_command(iter, strings)?;
    let directive = store_str(strings, &directive);
    let mut arguments = vec![];
    loop {
        if !matches!(iter.t, Token::Colon(..)) {
            break;
        };
        iter.next();
        match iter.t {
            Token::Word(..) => {
                let text = parse_argument_word(iter, strings)?;
                arguments.push(text.into());
            }
            Token::Quote(..) => {
                let text = parse_quote(iter, strings)?;
                arguments.push(text.into());
            }
            Token::LeftBracket(..) => {
                let argument = parse_bracket_component(iter, strings)?;
                arguments.push(argument.into());
            }
            Token::LeftSquare(..) => {
                let table = parse_table_component(iter, strings)?;
                arguments.push(table.into());
            }
            Token::LeftAngle(..) => {
                let command = parse_command_argument(iter, strings)?;
                arguments.push(command.into());
            }
            Token::Diamond(..) => {
                iter.next();
                if !matches!(iter.t, Token::Colon(..)) {
                    return iter.expectation_error(&[TokenType::Colon]);
                };
                iter.next();
                let right_hand_command = parse_command_expression(iter, strings)?;
                arguments.push(right_hand_command.into());
            }
            _ => {
                return iter.expectation_error(&[TokenType::Word, TokenType::Quote, TokenType::LeftBracket, TokenType::LeftSquare, TokenType::LeftAngle, TokenType::Diamond]);
            }
        };
    };
    let to = iter.position();
    Ok(ParsedDirective { directive, attributes, arguments, from, to })
}

/// Parse a sentence.
///
/// Recognizes `<sentence>`.
///
/// Corresponds to any combination of Word, Query, HashQuery, Whitespace tokens where
/// Whitespace is not at the beginning or end.
///
/// Assumes that the current token is Word, Query or HashQuery.
fn parse_sentence(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedText> {
    let sentence = String::new();
    let from = iter.position();
    match iter.t {
        Token::Word(..) => parse_sentence_with(iter, strings, from, sentence),
        _ => return iter.expectation_error(&[TokenType::Word]),
    }
}

/// Parse a sentence with an initial state. XX
///
/// Recognizes `<sentence>`.
fn parse_sentence_with(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>, from: Position, with: String) -> Result<ParsedText> {
    let mut sentence = with;
    loop {
        match iter.t {
            Token::Word(.., w) => {
                sentence.push_str(&w);
                iter.next();
            }
            Token::Whitespace(..) => {
                match iter.peek_next_glyph_token() {
                    Token::Word(..) => {
                        sentence.push(' ');
                        iter.skip_whitespace();
                    },
                    Token::Whitespace(..) => unreachable!(),
                    _ => break,
                }
            }
            _ => break,
        };
    };
    let to = iter.position();
    let str = store_str(strings, &sentence);
    Ok(ParsedText { str, from, to })
}

/// Parse a word as text.
fn parse_text_word(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedText> {
    let (word, from, to) = parse_word(iter, strings)?;
    let str = store_str(strings, &word);
    Ok(ParsedText { str, from, to })
}

/// Parse a word.
///
/// Corresponds to any combination of Word, Query, HashQuery tokens.
///
/// Used for attribute key, attribute value, directive header, dictionary key,
fn parse_word(iter: &mut TokenIter, _strings: &mut HashSet<Rc<str>>) -> Result<(String, Position, Position)> {
    let mut word = String::new();
    let from = iter.position();
    if !matches!(iter.t, Token::Word(..)) {
        return iter.expectation_error(&[TokenType::Word]);
    };
    loop {
        match iter.t {
            Token::Word(.., w) => {
                word.push_str(w);
                iter.next();
            }
            _ => break,
        };
    };
    let to = iter.position();
    Ok((word, from, to))
}

/// Parse an argument word.
///
/// Recognizes `<arg-word>`.
fn parse_argument_word(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedText> {
    let mut word = String::new();
    let from = iter.position();
    if !matches!(iter.t, Token::Word(..)) {
        return iter.expectation_error(&[TokenType::Word]);
    };
    loop {
        match iter.t {
            Token::Word(.., str) => {
                iter.next();
                word.push_str(str);
            }
            _ => break,
        };
    };
    let to = iter.position();
    let str = store_str(strings, &word);
    Ok(ParsedText { str, from, to })
}

/// Parse a quote.
///
/// Recognizes `<quote>`.
fn parse_quote(iter: &mut TokenIter, strings: &mut HashSet<Rc<str>>) -> Result<ParsedText> {
    match iter.t {
        Token::Quote(from, str) => {
            iter.next();
            let to = iter.position();
            let str = store_str(strings, str);
            Ok(ParsedText { str, from: *from, to })
        }
        _ => iter.expectation_error(&[TokenType::Quote]),
    }
}

//// Utility

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
    ExpectedClosingBracket(Position),
    ExpectedSequenceClosing(Position),
    ExpectedClosingAngularBracket(Position),
    ExpectedClosingSquare(Position),
    ExpectedColonAfterPrecedenceOperator(Position),
    ExpectedDirectiveAfterPrecedenceOperator(Position),

    ExpectedDirectiveClosing(Position),
    ExpectedDirectiveArgument(Position),

    ExpectedDirectiveKey(Position),
    ExpectedAttributeArgument(Position),

    ExpectedEntrySeparator(Position),
    ExpectedEnd(Position),
    CommentedBracket(Position),
    UnclosedQuote(Position),
    ExpectedClosingTag(Position, String),
    /// Directive of opening and closing tags do not match.
    MismatchedClosingTag(Position, String, Position, String),
    /// Tags cannot be used as arguments.
    IllegalTagDirectiveArgument(Position),
    IllegalTagAttribute(Position),
    ExpectedDictionaryClosing(Position),
    TooManyColumns(Position),
    TooFewColumns(Position),
    /// The concatenated table has a different number of columns.
    MismatchedColumns(Position, usize, usize),
    ExpectedBar(Position),
    ZeroColumns(Position),
    ExpectedColumns(Position, usize, usize),
    ExpectedBarOrSemicolon(Position),
    ExpectedTableEntry(Position),
    /// The token sequence is in invalid order.
    InvalidTokenSequence(Position),
    /// Composition of commands not allowed in tag arguments.
    CompositionNotAllowedInTag(Position),
    ExpectedWord(Position),
    /// Expected X but found Y at Z.
    Expected(&'static[TokenType], TokenType, Position),
    UnknownEscapeSequence(Position),
    UnknownHashSequence(Position),
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
        ParseError::ExpectedClosingBracket(at) => {
            format!("Expected bracket closing at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedSequenceClosing(at) => {
            format!("Expected sequence closing at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedClosingAngularBracket(at) => {
            format!("Expected closing angular bracket at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedClosingSquare(at) => {
            format!("Expected closing crotchet at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedColonAfterPrecedenceOperator(at) => {
            format!("Expected colon after grouping operator at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedDirectiveAfterPrecedenceOperator(at) => {
            format!("Expected command after grouping operator at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedDirectiveClosing(at) => {
            format!("Expected command closing at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedDirectiveArgument(at) => {
            format!("Expected command argument at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedDirectiveKey(at) => {
            format!("Expected command key at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedAttributeArgument(at) => {
            format!("Expected attribute value at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedEntrySeparator(at) => {
            format!("Expected entry separator at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedEnd(at) => {
            format!("Expected EOS at {}:{}.", at.line, at.column)
        }
        ParseError::CommentedBracket(at) => {
            format!("Commented bracket not allowed at {}:{}.", at.line, at.column)
        }
        ParseError::UnclosedQuote(at) => {
            format!("Unclosed quote at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedClosingTag(at, directive) => {
            format!("Expected closing tag matching directive \"{}\" at {}:{}.", directive, at.line, at.column)
        }
        ParseError::MismatchedClosingTag(opening_tag_at, directive, closing_tag_at, mismatch) => {
            format!("Closing tag directive \"{}\" at {}:{} does not match opening tag \"{}\" at {}:{}.", mismatch, closing_tag_at.line, closing_tag_at.column, directive, opening_tag_at.line, opening_tag_at.column)
        }
        ParseError::IllegalTagDirectiveArgument(at) => {
            format!("Illegal tag as directive argument at {}:{}.", at.line, at.column)
        }
        ParseError::IllegalTagAttribute(at) => {
            format!("Tag is not allowed as attribute at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedDictionaryClosing(at) => {
            format!("Expected dictionary closing \":}}\" at {}:{}.", at.line, at.column)
        }
        ParseError::TooManyColumns(at) => {
            format!("Row has too many columns at {}:{}.", at.line, at.column)
        }
        ParseError::TooFewColumns(at) => {
            format!("Row has too few columns at {}:{}.", at.line, at.column)
        }
        ParseError::MismatchedColumns(at, c, columns) => {
            format!("Cannot append table with {} columns to previous tables with {} columns at {}:{}.", c, columns, at.line, at.column)
        }
        ParseError::ExpectedBar(at) => {
            format!("Expected vertical bar after entry at {}:{}.", at.line, at.column)
        }
        ParseError::ZeroColumns(at) => {
            format!("Row with zero columns at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedColumns(at, c, columns) => {
            format!("Expected {} columns but found {} at {}:{}.", columns, c, at.line, at.column)
        }
        ParseError::ExpectedBarOrSemicolon(at) => {
            format!("Expected a bar or a semicolon at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedTableEntry(at) => {
            format!("Expected a table entry at {}:{}.", at.line, at.column)
        }
        ParseError::InvalidTokenSequence(at) => {
            format!("Found an unexpected token at {}:{}.", at.line, at.column)
        }
        ParseError::CompositionNotAllowedInTag(at) => {
            format!("Composition of commands not allowed in tag arguments at {}:{}.", at.line, at.column)
        }
        ParseError::ExpectedWord(at) => {
            format!("Expected Word token at {}:{}.", at.line, at.column)
        }
        ParseError::Expected(expected, found, at) => {
            format!("Expected {} but found {} at {}:{}.", list(expected), found, at.line, at.column)
        }
        ParseError::UnknownEscapeSequence(at) => {
            format!("Encountered unknown escape sequence at {}:{}.", at.line, at.column)
        }
        ParseError::UnknownHashSequence(at) => {
            format!("Encountered unknown hash sequence at {}:{}.", at.line, at.column)
        }
    }
}

fn list(expected: &[TokenType]) -> String {
    let mut str = String::new();
    let mut iter = expected.iter();
    if let Some(s) = iter.next() {
        str.push_str(&format!("{}", s));
    };
    while let Some(s) = iter.next() {
        str.push_str(&format!(" | {}", s));
    };
    str
}

//// Parsing results
////
//// Parsing a document yields a nested structure consisting of these structures.

//// Expression

#[derive(RefCast, Clone)]
#[repr(transparent)]
pub struct ParsedExpression(ParsedComponent);

impl ParsedExpression {

    fn as_component(&self) -> &ParsedComponent {
        &self.0
    }

}

impl Expression<Self, ParsedText, ParsedDictionary, ParsedTable, ParsedDirective, ParsedComponent> for ParsedExpression {

    type ComponentIterator<'b> = ComponentIterator<'b>;

    //type ComponentIterator<'b> =  where Self: 'b, 'b: 'a;

    type ComponentIteratorWithWhitespace<'b> = ComponentIteratorWithWhitespace<'b>;

    fn length(&self) -> usize {
        match &self.0 {
            ParsedComponent::Empty(..) => 0,
            ParsedComponent::Text(ParsedText { .. }) => 1,
            ParsedComponent::Table(ParsedTable { .. }) => 1,
            ParsedComponent::Dictionary(ParsedDictionary { .. }) => 1,
            ParsedComponent::Directive(ParsedDirective { .. }) => 1,
            ParsedComponent::Compound(components, ..) => components.len(),
        }
    }

    fn is_empty(&self) -> bool {
        matches!(self.0, ParsedComponent::Empty(..))
    }

    fn is_unary(&self) -> bool {
        matches!(self.0, ParsedComponent::Text(..) | ParsedComponent::Table(..) | ParsedComponent::Dictionary(..) | ParsedComponent::Directive(..))
    }

    fn is_compound(&self) -> bool {
        matches!(self.0, ParsedComponent::Compound(..))
    }

    fn get(&self, index: usize) -> Option<&ParsedComponent> {
        match &self.0 {
            ParsedComponent::Empty(..) => None,
            c @ ParsedComponent::Text(..) => {
                if index == 0 { Some(c) } else { None }
            }
            c @ ParsedComponent::Table(..) => {
                if index == 0 { Some(c) } else { None }
            }
            c @ ParsedComponent::Dictionary(..) => {
                if index == 0 { Some(c) } else { None }
            }
            c @ ParsedComponent::Directive(..) => {
                if index == 0 { Some(c) } else { None }
            }
            ParsedComponent::Compound(components, ..) => {
                components.get(index)
            }
        }
    }

    fn iter_components_with_whitespace(&self) -> Self::ComponentIteratorWithWhitespace<'_> {
        match &self.0 {
            ParsedComponent::Empty(..) => ComponentIteratorWithWhitespace::Empty,
            ParsedComponent::Text(..) | ParsedComponent::Dictionary(..) | ParsedComponent::Table(..) | ParsedComponent::Directive(..) => {
                ComponentIteratorWithWhitespace::Unary { component: &self.0, done: false }
            }
            ParsedComponent::Compound(components, whitespace, _from, _to) => {
                ComponentIteratorWithWhitespace::Compound { components, whitespace, index: 0, after_component: false }
            }
        }
    }

    fn iter_components(&self) -> Self::ComponentIterator<'_> {
        match &self.0 {
            ParsedComponent::Empty(..) => ComponentIterator::Empty,
            ParsedComponent::Text(..) | ParsedComponent::Table(..) | ParsedComponent::Dictionary(..) | ParsedComponent::Directive(..) => {
                ComponentIterator::Unary { component: &self.0, done: false }
            },
            ParsedComponent::Compound(components, _, _, _) => ComponentIterator::Compound { components, index: 0 },
        }
    }

    fn conform_text(&self) -> Option<&ParsedText> {
        if let ParsedComponent::Text(t) = self.as_component() {
            Some(t)
        } else {
            None
        }
    }

    fn conform_table(&self) -> Option<&ParsedTable> {
        if let ParsedComponent::Table(t) = self.as_component() {
            Some(t)
        } else {
            None
        }
    }

    fn conform_dictionary(&self) -> Option<&ParsedDictionary> {
        if let ParsedComponent::Dictionary(d) = self.as_component() {
            Some(d)
        } else {
            None
        }
    }

    fn conform_directive(&self) -> Option<&ParsedDirective> {
        if let ParsedComponent::Directive(d) = self.as_component() {
            Some(d)
        } else {
            None
        }
    }

    fn as_component(&self) -> &ParsedComponent {
        &self.0
    }

    fn is_text(&self) -> bool {
        matches!(self.0, ParsedComponent::Text(..))
    }

    fn is_table(&self) -> bool {
        matches!(self.0, ParsedComponent::Table(..))
    }

    fn is_dictionary(&self) -> bool {
        matches!(self.0, ParsedComponent::Dictionary(..))
    }

    fn is_directive(&self) -> bool {
        matches!(self.0, ParsedComponent::Directive(..))
    }

}

pub enum ComponentIterator<'a> {
    Empty,
    Unary { component: &'a ParsedComponent, done: bool },
    Compound { components: &'a[ParsedComponent], index: usize },
}

impl <'b> Iterator for ComponentIterator<'b> {

    type Item = &'b ParsedComponent;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ComponentIterator::Empty => None,
            ComponentIterator::Unary { component, done } => {
                if !*done {
                    *done = true;
                    Some(component)
                } else {
                    None
                }
            }
            ComponentIterator::Compound { components, index } => {
                if *index < components.len() {
                    let component = &components.get(*index).unwrap();
                    *index += 1;
                    Some(component)
                } else {
                    None
                }
            }
        }
    }

}

pub enum ComponentIteratorWithWhitespace<'b> {
    Empty,
    Unary { component: &'b ParsedComponent, done: bool },
    Compound { components: &'b[ParsedComponent], whitespace: &'b[bool], index: usize, after_component: bool },
}

impl <'b> Iterator for ComponentIteratorWithWhitespace<'b> {

    type Item = WhitespaceOption<&'b ParsedComponent>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ComponentIteratorWithWhitespace::Empty => None,
            ComponentIteratorWithWhitespace::Unary { component, done } => {
                if !*done {
                    *done = true;
                    Some(WhitespaceOption::Component(component))
                } else {
                    None
                }
            },
            ComponentIteratorWithWhitespace::Compound { components, whitespace, index, after_component } => {
                if *index != components.len() - 1 {
                    if *after_component {
                        let whitespace = whitespace[*index];
                        *index += 1;
                        if whitespace {
                            *after_component = false;
                            return Some(WhitespaceOption::Whitespace);
                        };
                    };
                } else {
                    if *after_component {
                        return None;
                    };
                };
                *after_component = true;
                Some(WhitespaceOption::Component(&components[*index]))
            }
        }
    }

}

impl From<ParsedText> for ParsedExpression {

    fn from(text: ParsedText) -> Self {
        ParsedComponent::Text(text).into_expression()
    }

}

impl From<ParsedTable> for ParsedExpression {

    fn from(sequence: ParsedTable) -> Self {
        ParsedComponent::Table(sequence).into_expression()
    }

}

impl From<ParsedDictionary> for ParsedExpression {

    fn from(dictionary: ParsedDictionary) -> Self {
        ParsedComponent::Dictionary(dictionary).into_expression()
    }

}

impl From<ParsedDirective> for ParsedExpression {

    fn from(command: ParsedDirective) -> Self {
        ParsedComponent::Directive(command).into_expression()
    }

}

impl From<ParsedComponent> for ParsedExpression {

    fn from(component: ParsedComponent) -> Self {
        component.into_expression()
    }

}

//// Text

#[derive(PartialEq, Eq, Clone)]
pub struct ParsedText { pub str: Rc<str>, pub from: Position, pub to: Position }

impl Text<ParsedExpression, Self, ParsedDictionary, ParsedTable, ParsedDirective, ParsedComponent> for ParsedText {

    fn as_str(&self) -> &str {
        &self.str
    }

}

//// Dictionary

/// A parsed dictionary.
#[derive(Clone)]
pub struct ParsedDictionary {
    pub entries: Vec<(Rc<str>, ParsedExpression)>,
    pub from: Position,
    pub to: Position,
}

impl Dictionary<ParsedExpression, ParsedText, Self, ParsedTable, ParsedDirective, ParsedComponent> for ParsedDictionary {

    type EntryIterator<'a> = EntryIterator<'a>;

    //type EntryIterator<'b> = EntryIterator<'b> where 'a: 'b, EntryIterator<'b>: 'b;

    fn length(&self) -> usize {
        self.entries.len()
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn get_by(&self, key: &str) -> Option<&ParsedExpression> {
        for entry in &self.entries {
            if key.eq(entry.0.deref()) {
                return Some(&entry.1);
            };
        };
        None
    }

    fn get_at(&self, index: usize) -> Option<(&str, &ParsedExpression)> {
        if let Some(entry) = self.entries.get(index) {
            Some((entry.0.borrow(), &entry.1))
        } else {
            None
        }
    }

    fn iter_entries(&self) -> Self::EntryIterator<'_> {
        EntryIterator { iter: self.entries.iter() }
    }

}

pub type EntryIterator<'a> = AttributeIterator<'a>;

impl ParsedDictionary {

    pub fn empty(from: Position, to: Position) -> Self {
        ParsedDictionary { entries: vec![], from, to }
    }

    pub fn get_entry(&self, key: &str) -> Option<(&str, &ParsedExpression)> {
        for entry in &self.entries {
            if key.eq(entry.0.deref()) {
                return Some((&entry.0, &entry.1));
            };
        }
        return None;
    }

    pub fn get(&self, key: &str) -> Option<&ParsedExpression> {
        for entry in &self.entries {
            if key.eq(entry.0.deref()) {
                return Some(&entry.1);
            };
        }
        return None;
    }

    pub fn size(&self) -> usize {
        self.entries.len()
    }

}

//// Table

/// A parsed table.
#[derive(Clone)]
pub struct ParsedTable {
    pub elements: Vec<ParsedExpression>,
    pub from: Position,
    pub to: Position,
    pub columns: usize,
}

impl Table<ParsedExpression, ParsedText, ParsedDictionary, ParsedTable, ParsedDirective, ParsedComponent> for ParsedTable {

    type RowIterator<'b> = RowIterator<'b>;

    type ListIterator<'b> = Iter<'b, ParsedExpression>;

    fn is_empty(&self) -> bool {
        self.elements.is_empty()
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

    fn size(&self) -> usize {
        self.elements.len()
    }

    fn get(&self, row: usize, column: usize) -> Option<&ParsedExpression> {
        self.elements.get(row * self.columns + column)
    }

    fn get_row(&self, row: usize) -> Option<&[ParsedExpression]> {
        if self.columns != 0 {
            Some(&self.elements[row * self.columns .. row * (self.columns + 1)])
        } else {
            None
        }
    }

    fn iter_rows(&self) -> Self::RowIterator<'_> {
        RowIterator { columns: self.columns, iter: self.elements.iter() }
    }

    fn is_list(&self) -> bool {
        self.columns <= 1
    }

    fn len_as_list(&self) -> usize {
        self.elements.len()
    }

    fn get_list_element(&self, index: usize) -> Option<&ParsedExpression> {
        self.elements.get(index)
    }

    fn iter_list_elements(&self) -> Self::ListIterator<'_> {
        self.elements.iter()
    }

    fn is_tuple(&self) -> bool {
        self.rows() <= 1
    }

}

impl ParsedTable {

    /// Empty table.
    pub fn empty(at: Position) -> Self {
        ParsedTable { elements: vec![], from: at, to: at, columns: 0 }
    }

}

pub struct RowIterator<'a> {
    columns: usize,
    iter: Iter<'a, ParsedExpression>,
}

impl <'a> Iterator for RowIterator<'a> {

    type Item = Box<[&'a ParsedExpression]>;

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

//// Directive

/// A parsed directive.
#[derive(Clone)]
pub struct ParsedDirective {
    pub directive: Rc<str>,
    pub attributes: Vec<(Rc<str>, ParsedExpression)>,
    pub arguments: Vec<ParsedComponent>,
    pub from: Position,
    pub to: Position,
}

impl Directive<ParsedExpression, ParsedText, ParsedDictionary, ParsedTable, Self, ParsedComponent> for ParsedDirective {

    type ArgumentIterator<'b> = Iter<'b, ParsedComponent>;

    type AttributeIterator<'b> = AttributeIterator<'b>;

    fn label(&self) -> &str {
        &self.directive
    }

    fn length(&self) -> usize {
        self.arguments.len()
    }

    fn has_attributes(&self) -> bool {
        !self.attributes.is_empty()
    }

    fn has_arguments(&self) -> bool {
        !self.arguments.is_empty()
    }

    fn get_argument(&self, index: usize) -> Option<&ParsedComponent> {
        self.arguments.get(index)
    }

    fn get_attribute(&self, key: &str) -> Option<&ParsedExpression> {
        for attribute in &self.attributes {
            if key.eq(attribute.0.deref()) {
                return Some(&attribute.1);
            };
        };
        None
    }

    fn get_attribute_at(&self, index: usize) -> Option<(&str, &ParsedExpression)> {
        if let Some(attribute) = self.attributes.get(index) {
            Some((attribute.0.borrow(), &attribute.1))
        } else {
            None
        }
    }

    fn iter_arguments(&self) -> Self::ArgumentIterator<'_> {
        self.arguments.iter()
    }

    fn iter_attributes(&self) -> Self::AttributeIterator<'_> {
        AttributeIterator { iter: self.attributes.iter() }
    }

}

pub struct AttributeIterator<'a> {
    iter: Iter<'a, (Rc<str>, ParsedExpression)>,
}

impl <'a> Iterator for AttributeIterator<'a> {

    type Item = (&'a str, &'a ParsedExpression);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(attribute) = self.iter.next() {
            Some((&attribute.0, &attribute.1))
        } else {
            None
        }
    }

}

impl ParsedDirective {

    pub fn length(&self) -> usize {
        self.arguments.len()
    }

    pub fn get(&self, index: usize) -> Option<&ParsedComponent> {
        self.arguments.get(index)
    }

}

//// Component

#[derive(Clone)]
pub enum ParsedComponent {
    Empty(Position, Position),
    Text(ParsedText),
    Table(ParsedTable),
    Dictionary(ParsedDictionary),
    Directive(ParsedDirective),
    Compound(Box<[ParsedComponent]>, Box<[bool]>, Position, Position),
}

impl ParsedComponent {

    pub fn empty(from: Position, to: Position) -> Self {
        ParsedComponent::Empty(from, to)
    }

    pub fn from_components(from: Position, to: Position, mut components: Vec<ParsedComponent>, whitespace: Vec<bool>) -> Self {
        let len = components.len();
        if len == 0 {
            ParsedComponent::Empty(from, to)
        } else if len == 1 {
            components.pop().unwrap()
        } else {
            assert_eq!(len - 1, whitespace.len());
            ParsedComponent::Compound(components.into_boxed_slice(), whitespace.into_boxed_slice(), from, to)
        }
    }

}

impl Component<ParsedExpression, ParsedText, ParsedDictionary, ParsedTable, ParsedDirective, ParsedComponent> for ParsedComponent {

    fn as_expression(&self) -> &ParsedExpression {
        ParsedExpression::ref_cast(self)
    }

    fn as_text(&self) -> Option<&ParsedText> {
        if let ParsedComponent::Text(t) = self {
            Some(t)
        } else {
            None
        }
    }

    fn as_table(&self) -> Option<&ParsedTable> {
        if let ParsedComponent::Table(t) = self {
            Some(t)
        } else {
            None
        }
    }

    fn as_dictionary(&self) -> Option<&ParsedDictionary> {
        if let ParsedComponent::Dictionary(d) = self {
            Some(d)
        } else {
            None
        }
    }

    fn as_directive(&self) -> Option<&ParsedDirective> {
        if let ParsedComponent::Directive(d) = self {
            Some(d)
        } else {
            None
        }
    }

    fn is_text(&self) -> bool {
        matches!(self, ParsedComponent::Text(..))
    }

    fn is_table(&self) -> bool {
        matches!(self, ParsedComponent::Table(..))
    }

    fn is_dictionary(&self) -> bool {
        matches!(self, ParsedComponent::Dictionary(..))
    }

    fn is_directive(&self) -> bool {
        matches!(self, ParsedComponent::Directive(..))
    }

}

impl ParsedComponent {

    pub fn from(&self) -> Position {
        match self {
            ParsedComponent::Empty(from, ..) => from,
            ParsedComponent::Text(ParsedText { from, .. }) => from,
            ParsedComponent::Table(ParsedTable { from, .. }) => from,
            ParsedComponent::Dictionary(ParsedDictionary { from, .. }) => from,
            ParsedComponent::Directive(ParsedDirective { from, .. }) => from,
            ParsedComponent::Compound(_, _, from, _) => from,
        }.clone()
    }

    pub fn to(&self) -> Position {
        match self {
            ParsedComponent::Empty(.., to) => to,
            ParsedComponent::Text(ParsedText { to, .. }) => to,
            ParsedComponent::Table(ParsedTable { to, .. }) => to,
            ParsedComponent::Dictionary(ParsedDictionary { to, .. }) => to,
            ParsedComponent::Directive(ParsedDirective { to, .. }) => to,
            ParsedComponent::Compound(_, _, _, to) => to,
        }.clone()
    }

    fn into_expression(self) -> ParsedExpression {
        ParsedExpression(self)
    }

}

impl From<ParsedExpression> for ParsedComponent {

    fn from(component: ParsedExpression) -> Self {
        component.0
    }

}

impl From<ParsedText> for ParsedComponent {

    fn from(text: ParsedText) -> Self {
        ParsedComponent::Text(text)
    }

}

impl From<ParsedTable> for ParsedComponent {

    fn from(sequence: ParsedTable) -> Self {
        ParsedComponent::Table(sequence)
    }

}

impl From<ParsedDictionary> for ParsedComponent {

    fn from(dictionary: ParsedDictionary) -> Self {
        ParsedComponent::Dictionary(dictionary)
    }

}

impl From<ParsedDirective> for ParsedComponent {

    fn from(command: ParsedDirective) -> Self {
        ParsedComponent::Directive(command)
    }

}

//// Parsing result formatting

#[derive(Clone, Copy)]
pub enum TokenType {
    Word, Quote,
    Colon, Semicolon, Bar, Diamond,
    LeftBracket, RightBracket, HashRightBracket,
    LeftSquare, RightSquare, HashRightSquare,
    LeftAngle, RightAngle,
    Whitespace, End,
}

impl Display for TokenType {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Word => write!(f, "Word"),
            TokenType::Quote => write!(f, "Quote"),
            TokenType::Colon => write!(f, ":"),
            TokenType::Semicolon => write!(f, ";"),
            TokenType::Bar => write!(f, "|"),
            TokenType::Diamond => write!(f, "<>"),
            TokenType::LeftBracket => write!(f, "{{"),
            TokenType::RightBracket => write!(f, "}}"),
            TokenType::HashRightBracket => write!(f, "#}}"),
            TokenType::LeftSquare => write!(f, "["),
            TokenType::RightSquare => write!(f, "]"),
            TokenType::HashRightSquare => write!(f, "#]"),
            TokenType::LeftAngle => write!(f, "<"),
            TokenType::RightAngle => write!(f, ">"),
            TokenType::Whitespace => write!(f, "Whitespace"),
            TokenType::End => write!(f, "End"),
        }
    }

}

impl Token {

    fn to_type(&self) -> TokenType {
        match self {
            Token::Word(..) => TokenType::Word,
            Token::Quote(..) => TokenType::Quote,
            Token::Colon(..) => TokenType::Colon,
            Token::Semicolon(..) => TokenType::Semicolon,
            Token::Bar(..) => TokenType::Bar,
            Token::Diamond(..) => TokenType::Diamond,
            Token::LeftBracket(..) => TokenType::LeftBracket,
            Token::RightBracket(..) => TokenType::RightBracket,
            Token::HashRightBracket(..) => TokenType::HashRightBracket,
            Token::LeftSquare(..) => TokenType::LeftSquare,
            Token::RightSquare(..) => TokenType::RightSquare,
            Token::HashRightSquare(..) => TokenType::HashRightSquare,
            Token::LeftAngle(..) => TokenType::LeftAngle,
            Token::RightAngle(..) => TokenType::RightAngle,
            Token::Whitespace(..) => TokenType::Whitespace,
            Token::End(..) => TokenType::End,
        }
    }

}

pub enum ParseTask {
    Expression, Dictionary, Table, Directive
}
