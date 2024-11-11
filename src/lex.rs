//! Khi lexer reference implementation.

use std::ops::Deref;
use crate::pdm::Position;
use crate::translate_escape_character;

//// Token

#[derive(PartialEq, Eq, Clone)]
pub enum Token {
    Whitespace(Position),
    Word(Position, String),
    Transcription(Position, String),
    TextBlock(Position, String),
    Colon(Position),
    Semicolon(Position),
    Bar(Position),
    Tilde(Position),
    DoubleArrow(Position),
    LeftBracket(Position),
    RightBracket(Position),
    LeftSquare(Position),
    RightSquare(Position),
    LeftAngle(Position),
    RightAngle(Position),
    End(Position),
}

impl Token {

    pub fn at(&self) -> Position {
        match self {
            Token::Whitespace(at) => *at,
            Token::Word(at, ..) => *at,
            Token::Transcription(at, ..) => *at,
            Token::TextBlock(at, ..) => *at,
            Token::Colon(at) => *at,
            Token::Semicolon(at) => *at,
            Token::Bar(at) => *at,
            Token::Tilde(at) => *at,
            Token::DoubleArrow(at) => *at,
            Token::LeftBracket(at) => *at,
            Token::RightBracket(at) => *at,
            Token::LeftSquare(at) => *at,
            Token::RightSquare(at) => *at,
            Token::LeftAngle(at) => *at,
            Token::RightAngle(at) => *at,
            Token::End(at) => *at,
        }
    }

}

//// Char iterator

pub struct CharIter<It: Iterator<Item = char>> {
    chars: It,
    c: Option<char>, // Current character
    d: Option<char>, // Next character
    e: Option<char>,
    index: usize,
    line: usize,
    column: usize,
}

impl <'a, It: Iterator<Item = char>> CharIter<It> {

    pub fn new(chars: It) -> Self {
        let mut iter = CharIter { chars, c: None, d: None, e: None, index: 0, line: 1, column: 1 };
        iter.next();
        iter.next();
        iter.next();
        iter
    }

    pub fn next(&mut self) {
        if let Some(c) = self.c {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            };
            self.index += 1;
        };
        self.c = self.d;
        self.d = self.e;
        loop {
            self.e = self.chars.next();
            if self.e != Some('\r') {
                break;
            }
        }
    }

    pub fn next_two(&mut self) {
        self.next();
        self.next();
    }

    pub fn position(&self) -> Position {
        Position {
            index: self.index,
            line: self.line,
            column: self.column,
        }
    }

    fn skip_line(&mut self) {
        loop {
            if let Some(c) = self.c {
                self.next();
                if c == '\n' {
                    break;
                };
            } else {
                break;
            }
        }
    }

}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n'
}

//// Lex

/// Iterates over characters and produces tokens.
pub fn lex<It: Iterator<Item = char>>(chars: It) -> Result<Vec<Token>, LexError> {
    let mut iter = CharIter::new(chars);
    let mut tokens = vec![];
    loop {
        if let Some(c) = iter.c {
            if is_whitespace(c) { // Whitespace
                let whitespace = lex_whitespace(&mut iter)?;
                tokens.push(whitespace);
            } else if c == ':' {
                if let Some(':') = iter.d { // Colon glyph
                    let word = lex_word(&mut iter)?;
                    tokens.push(word);
                } else { // Colon
                    tokens.push(Token::Colon(iter.position()));
                    iter.next();
                }
            } else if c == ';' { // Semicolon
                if let Some(';') = iter.d { // Semicolon glyph
                    let word = lex_word(&mut iter)?;
                    tokens.push(word);
                } else { // Semicolon
                    tokens.push(Token::Semicolon(iter.position()));
                    iter.next();
                }
            } else if c == '|' {
                if let Some('|') = iter.d { // Bar glyph
                    let word = lex_word(&mut iter)?;
                    tokens.push(word);
                } else { // Bar
                    tokens.push(Token::Bar(iter.position()));
                    iter.next();
                }
            } else if c == '~' {
                if let Some('~') = iter.d { // Tilde glyph
                    let word = lex_word(&mut iter)?;
                    tokens.push(word);
                } else { // Tilde
                    tokens.push(Token::Tilde(iter.position()));
                    iter.next();
                }
            } else if c == '`' { // Illegal escape character
                let word = lex_word(&mut iter)?;
                tokens.push(word);
            } else if c == '\\' { // Transcription
                let transcription = lex_transcription(&mut iter)?;
                tokens.push(transcription);
            } else if c == '{' { // Left bracket
                tokens.push(Token::LeftBracket(iter.position()));
                iter.next();
            } else if c == '}' { // Right bracket
                tokens.push(Token::RightBracket(iter.position()));
                iter.next();
            } else if c == '[' { // Left square
                tokens.push(Token::LeftSquare(iter.position()));
                iter.next();
            } else if c == ']' { // Right square
                tokens.push(Token::RightSquare(iter.position()));
                iter.next();
            } else if c == '<' {
                if let Some(d) = iter.d {
                    if d == '<' { // Left angle glyph
                        let token = lex_word(&mut iter)?;
                        tokens.push(token);
                    } else if d == '#' { // Text block
                        let text_block = lex_text_block(&mut iter)?;
                        tokens.push(text_block);
                    } else { // Left angle
                        tokens.push(Token::LeftAngle(iter.position()));
                        iter.next();
                    };
                } else { // Left angle
                    tokens.push(Token::LeftAngle(iter.position()));
                    iter.next();
                }
            } else if c == '>' {
                if let Some('>') = iter.d { // Right angle glyph
                    let token = lex_word(&mut iter)?;
                    tokens.push(token);
                } else { // Right angle
                    tokens.push(Token::RightAngle(iter.position()));
                    iter.next();
                }
            } else if c == '#' {
                if let Some(d) = iter.d {
                    if d == '#' || is_whitespace(d) { // Comment
                        let whitespace = lex_whitespace(&mut iter)?;
                        tokens.push(whitespace);
                    } else { // Hash glyph: handle illegal cases in word
                        let word = lex_word(&mut iter)?;
                        tokens.push(word);
                    }
                } else { // Comment before end
                    let whitespace = lex_whitespace(&mut iter)?;
                    tokens.push(whitespace);
                }
            } else if c == '=' && iter.d == Some('>') && iter.e != Some('>') {
                tokens.push(Token::DoubleArrow(iter.position()));
                iter.next(); iter.next();
            } else { // Text glyph
                let word = lex_word(&mut iter)?;
                tokens.push(word);
            }
        } else {
            tokens.push(Token::End(iter.position()));
            break;
        }
    }
    Ok(tokens)
}

/// Lex whitespace, including comments
///
/// Assumes that the current character is whitespace or a hash opening a comment.
fn lex_whitespace<It: Iterator<Item = char>>(iter: &mut CharIter<It>) -> Result<Token, LexError> {
    let at = iter.position();
    loop {
        if let Some(c) = iter.c {
            if is_whitespace(c) {
                iter.next();
            } else if c == '#' {
                if let Some(d) = iter.d {
                    if is_whitespace(d) || d == '#' {
                        iter.skip_line();
                    } else {
                        break;
                    };
                } else { // EOS
                    iter.next();
                    break;
                };
            } else {
                break;
            };
        } else {
            break;
        };
    };
    Ok(Token::Whitespace(at))
}

/// Lex a word.
///
/// Assumes that the current character is a glyph.
fn lex_word<It: Iterator<Item = char>>(iter: &mut CharIter<It>) -> Result<Token, LexError> {
    let at = iter.position();
    let mut string = String::new();
    loop {
        if let Some(c) = iter.c {
            if is_whitespace(c) { // Whitespace
                break;
            } else if c == '\\' || c == '{' || c == '}' || c == '[' || c == ']' { // Reserved
                break;
            } else if c == ':' || c == ';' || c == '|' || c == '~' || c == '<' || c == '>' {
                if let Some(d) = iter.d {
                    if d == c { // Repeated escape sequence
                        iter.next(); iter.next();
                        string.push(c);
                    } else { // Reserved
                        break;
                    }
                } else { // Reserved
                    break;
                }
            } else if c == '`' { // Character escape character
                if let Some(d) = iter.d { // Escape sequence
                    let x = match translate_escape_character(d) {
                        Ok(x) => x,
                        Err(..) => return Err(LexError::InvalidEscapeSequence(iter.position())),
                    };
                    iter.next_two();
                    string.push(x);
                } else {
                    return Err(LexError::EscapeEos);
                }
            } else if c == '#' {
                if let Some(d) = iter.d {
                    if d == '#' || is_whitespace(d) { // Comment
                        break;
                    } else if d == '\\' || d == '{' || d == '}' || d == '[' || d == ']' { // Following reserved character
                        return Err(LexError::InvalidHashSequence(iter.position()));
                    } else if d == ':' || d == ';' || d == '|' || d == '~' || d == '<' || d == '>' {
                        if iter.e == Some(d) { // Following repeated escape sequence
                            iter.next();
                            string.push('#');
                        } else { // Following reserved character
                            return Err(LexError::InvalidHashSequence(iter.position()));
                        }
                    } else { // # Hash glyph
                        iter.next();
                        string.push('#');
                    }
                } else {
                    break;
                };
            } else if c == '=' && iter.d == Some('>') && iter.e != Some('>') { // =>
                break;
            } else { // Glyph
                iter.next();
                string.push(c);
            };
        } else { // End
            break;
        };
    };
    Ok(Token::Word(at, string))
}

/// Lex a transcription.
///
/// Assumes that the current character is `\ `.
fn lex_transcription<It: Iterator<Item = char>>(iter: &mut CharIter<It>) -> Result<Token, LexError> {
    let at = iter.position();
    let mut string = String::new();
    iter.next();
    loop {
        if let Some(c) = iter.c {
            if c == '\\' || c == '\n' {
                iter.next();
                break;
            } else if c == '`' {
                if let Some(d) = iter.d {
                    let e = match translate_escape_character(d) {
                        Ok(e) => e,
                        Err(..) => return Err(LexError::InvalidEscapeSequence(iter.position())),
                    };
                    iter.next_two();
                    string.push(e);
                } else {
                    return Err(LexError::EscapeEos);
                };
            } else {
                iter.next();
                string.push(c);
            };
        } else {
            break;
        };
    };
    Ok(Token::Transcription(at, string))
}

/// Lex a text block.
///
/// Assumes that the current characters are `<#`.
fn lex_text_block<It: Iterator<Item = char>>(iter: &mut CharIter<It>) -> Result<Token, LexError> {
    let at = iter.position();
    let mut closing_tag = String::new();
    let mut configuration = vec![Flag::Footer, Flag::Header, Flag::Excess];
    let mut content = String::new();
    iter.next_two();
    closing_tag.push('<');
    closing_tag.push('#');
    'tag: loop {
        if let Some(c) = iter.c {
            if c == '>' {
                iter.next();
                closing_tag.push('>');
                break;
            } else if is_whitespace(c) {
                closing_tag.push('>');
                skip_whitespace_in_text_block_tag(iter, at)?;
                loop { // Read configuration
                    if let Some(c) = iter.c {
                        if c == 'f' {
                            iter.next(); configuration.push(Flag::Footer);
                        } else if c == 'h' {
                            iter.next(); configuration.push(Flag::Header);
                        } else if c == 'x' {
                            iter.next(); configuration.push(Flag::Excess);
                        } else if c == 't' {
                            iter.next(); configuration.push(Flag::Trailing);
                        } else if c == 'l' {
                            iter.next(); configuration.push(Flag::Leading);
                        } else if c == 'n' {
                            iter.next(); configuration.push(Flag::Newline);
                        } else if c == 'r' {
                            iter.next(); configuration.clear();
                        } else if c == '>' {
                            iter.next();
                            break 'tag;
                        } else if is_whitespace(c) {
                            break;
                        } else {
                            return Err(LexError::InvalidTextBlockConfiguration(iter.position()))
                        }
                    } else {
                        return Err(LexError::UnclosedTextBlock(iter.position()));
                    }
                }
                skip_whitespace_in_text_block_tag(iter, at)?;
                if let Some(c) = iter.c {
                    if c == '>' {
                        iter.next();
                        break 'tag;
                    } else {
                        return Err(LexError::UnclosedTextBlock(iter.position()));
                    }
                }
            } else {
                iter.next();
                closing_tag.push(c);
            }
        } else {
            return Err(LexError::UnclosedTextBlock(iter.position()));
        }
    }
    loop { // Read content.
        if let Some(c) = iter.c {
            content.push(c);
            iter.next();
            if content.ends_with(closing_tag.deref()) {
                content = content.replace(closing_tag.deref(), "");
                break;
            }
        } else {
            return Err(LexError::UnclosedTextBlock(iter.position()));
        }
    }
    for flag in configuration {
        content = match flag {
            Flag::Footer => delete_blank_footer(content),
            Flag::Header => delete_blank_header(content),
            Flag::Excess => delete_excess_indentation(content),
            Flag::Trailing => delete_trailing_whitespace(content),
            Flag::Leading => delete_leading_whitespace(content),
            Flag::Newline => delete_newlines(content),
        }
    }
    Ok(Token::TextBlock(at, content))
}

enum Flag { Footer, Header, Excess, Trailing, Leading, Newline }

fn skip_whitespace_in_text_block_tag<It: Iterator<Item = char>>(iter: &mut CharIter<It>, at: Position) -> Result<(), LexError> {
    loop { // Skip whitespace
        iter.next();
        if let Some(c) = iter.c {
            if !is_whitespace(c) {
                return Ok(());
            }
        } else {
            return Err(LexError::UnclosedTextBlock(at));
        }
    }
}

pub enum LexError {
    /// Tried to escape EOS.
    EscapeEos,
    /// Character escape sequence is not recognized.
    InvalidEscapeSequence(Position),
    /// Illegal character after hash.
    InvalidHashSequence(Position),
    /// Text block was never closed.
    UnclosedTextBlock(Position),
    /// Invalid text block configuration.
    InvalidTextBlockConfiguration(Position),
}

//// Strings

fn delete_blank_footer(string: String) -> String {
    let mut string = string.into_bytes();
    let mut i = string.len();
    while i > 0 {
        i -= 1; let c = string[i];
        if c == ' ' as u8 || c == '\t' as u8 {
            continue;
        } else if c == '\n' as u8 {
            string.truncate(i + 1);
            break;
        } else {
            break;
        }
    }
    String::from_utf8(string).unwrap()
}

fn delete_blank_header(string: String) -> String {
    let mut string = string.into_bytes();
    let len = string.len();
    let mut i = 0;
    let mut j = 0;
    while i < len {
        let c = string[i]; i += 1;
        if c == '\n' as u8 {
            j = 0;
            break;
        } else if c == ' ' as u8 || c == '\t' as u8 {
            string[j] = c; j += 1;
        } else {
            string[j] = c; j += 1;
            break;
        }
    }
    while i < len {
        let c = string[i]; i += 1;
        string[j] = c; j += 1;
    }
    string.truncate(j);
    String::from_utf8(string).unwrap()
}

fn delete_excess_indentation(string: String) -> String {
    let mut string = string.into_bytes();
    let len = string.len();
    let mut r = len;
    while r > 0 {
        r -= 1; let c = string[r];
        if c == '\n' as u8 {
            break;
        }
    }
    let mut indentation = vec![];
    let mut i = 0;
    while i < r {
        let c = string[i];
        if c == ' ' as u8 || c == '\t' as u8 {
            indentation.push(c);
            i += 1;
        } else {
            skip_to_next_line(&string, r, &mut i);
            break;
        }
    }
    while i < r {
        let x = indentation.len();
        let mut j = 0;
        while j < x {
            let c = string[i];
            if c == indentation[j] {
                i += 1;
                j += 1;
            } else {
                indentation.truncate(j);
                break;
            }
        }
        skip_to_next_line(&string, r, &mut i);
    }
    let excess = indentation.len();
    if excess > 0 {
        let mut i = 0;
        let mut j = 0;
        while i < r {
            i += excess;
            copy_line(&mut string, len, &mut i, &mut j);
        }
        while i < len {
            let c = string[i]; i += 1;
            string[j] = c; j += 1;
        }
        string.truncate(j);
    }
    String::from_utf8(string).unwrap()
}

fn skip_to_next_line(string: &Vec<u8>, len: usize, i: &mut usize) {
    while *i < len {
        let c = string[*i]; *i += 1;
        if c == '\n' as u8 {
            break;
        }
    }
}

fn delete_trailing_whitespace(string: String) -> String {
    fn backtrack(string: &[u8], j: &mut usize) {
        while *j > 0 {
            let d = string[*j - 1];
            if d == ' ' as u8 || d == '\t' as u8 {
                *j -= 1;
            } else {
                break;
            }
        }
    }
    let mut string = string.into_bytes();
    let len = string.len();
    let mut i = 0;
    let mut j = 0;
    while i < len {
        let c = string[i]; i += 1;
        if c == '\n' as u8 {
            backtrack(&string, &mut j);
            string[j] = '\n' as u8; j += 1;
        } else {
            string[j] = c; j += 1;
        }
    }
    backtrack(&string, &mut j);
    string.truncate(j);
    String::from_utf8(string).unwrap()
}

fn delete_leading_whitespace(string: String) -> String {
    let mut string = string.into_bytes();
    let len = string.len();
    let mut i = 0;
    let mut j = 0;
    while i < len {
        let c = string[i];
        if c != ' ' as u8 && c != '\t' as u8 {
            copy_line(&mut string, len, &mut i, &mut j);
        } else {
            i += 1;
        }
    }
    string.truncate(j);
    String::from_utf8(string).unwrap()
}

fn delete_newlines(string: String) -> String {
    let mut string = string.into_bytes();
    let len = string.len();
    let mut i = 0;
    let mut j = 0;
    while i < len {
        let c = string[i]; i += 1;
        if c != '\n' as u8 {
            string[j] = c; j += 1;
        }
    }
    string.truncate(j);
    String::from_utf8(string).unwrap()
}

fn copy_line(string: &mut [u8], len: usize, i: &mut usize, j: &mut usize) {
    while *i < len {
        let d = string[*i]; *i += 1;
        string[*j] = d; *j += 1;
        if d == '\n' as u8 {
            break;
        }
    }
}
