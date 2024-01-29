use std::ops::Deref;
use crate::translate_escape_character;

//// Position

/// A char position.
///
/// Contains a line number and a column number, corresponding to a character in a
/// document.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Position { pub index: usize, pub line: usize, pub column: usize }

//// Token

#[derive(PartialEq, Eq, Clone)]
pub enum Token {
    Whitespace(Position),
    Word(Position, String),
    Quotation(Position, String),
    TextBlock(Position, String),
    Colon(Position),
    Semicolon(Position),
    Bar(Position),
    Tilde(Position),
    Diamond(Position),
    LeftBracket(Position),
    RightBracket(Position),
    LeftSquare(Position),
    RightSquare(Position),
    LeftAngle(Position),
    RightAngle(Position),
    End(Position),
}

impl Token {

    pub fn position(&self) -> Position {
        match self {
            Token::Whitespace(at) => *at,
            Token::Word(at, ..) => *at,
            Token::Quotation(at, ..) => *at,
            Token::TextBlock(at, ..) => *at,
            Token::Colon(at) => *at,
            Token::Semicolon(at) => *at,
            Token::Bar(at) => *at,
            Token::Tilde(at) => *at,
            Token::Diamond(at) => *at,
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
    index: usize,
    line: usize,
    column: usize,
}

impl <'a, It: Iterator<Item = char>> CharIter<It> {

    pub fn new(mut chars: It) -> Self {
        let c = chars.next();
        let d = chars.next(); // TODO: Carriage return delete
        CharIter { chars, c, d, index: 0, line: 1, column: 1 }
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
        self.d = self.chars.next(); // TODO: Carriage return delete
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

//// Lex

/// Iterates over characters and produces tokens.
pub fn lex<It: Iterator<Item = char>>(chars: It) -> Result<Vec<Token>, LexError> {
    let mut iter = CharIter::new(chars);
    let mut tokens = vec![];
    loop {
        if let Some(c) = iter.c {
            if c == '{' {
                let at = iter.position();
                iter.next();
                tokens.push(Token::LeftBracket(at));
            } else if c == '}' {
                let at = iter.position();
                iter.next();
                tokens.push(Token::RightBracket(at));
            } else if c == '[' {
                let at = iter.position();
                iter.next();
                tokens.push(Token::LeftSquare(at));
            } else if c == ']' {
                let at = iter.position();
                iter.next();
                tokens.push(Token::RightSquare(at));
            } else if c == '<' {
                if let Some(d) = iter.d {
                    if d == '<' { // Repeated escape sequence
                        let token = lex_word(&mut iter)?;
                        tokens.push(token);
                    } else if d == '>' { // Diamond
                        let at = iter.position();
                        iter.next_two();
                        tokens.push(Token::Diamond(at));
                    } else if d == '#' { // Text block
                        let at = iter.position();
                        let quote = lex_text_block(&mut iter, at)?;
                        tokens.push(quote);
                    } else { // Left angle
                        let at = iter.position();
                        iter.next();
                        tokens.push(Token::LeftAngle(at));
                    };
                } else { // Left angle
                    let at = iter.position();
                    iter.next();
                    tokens.push(Token::LeftAngle(at));
                };
            } else if c == '>' {
                if let Some('>') = iter.d { // Repeated escape sequence
                    let token = lex_word(&mut iter)?;
                    tokens.push(token);
                } else { // Right angle
                    let at = iter.position();
                    iter.next();
                    tokens.push(Token::RightAngle(at));
                };
            } else if c == '"' { // Quotation
                let token = lex_quotation(&mut iter)?;
                tokens.push(token);
            } else if c == ':' {
                if let Some(':') = iter.d { // Repeated escape sequence
                    let token = lex_word(&mut iter)?;
                    tokens.push(token);
                } else { // Colon
                    let at = iter.position();
                    iter.next();
                    tokens.push(Token::Colon(at));
                };
            } else if c == ';' {
                if let Some(';') = iter.d { // Repeated escape sequence
                    let token = lex_word(&mut iter)?;
                    tokens.push(token);
                } else { // Semicolon
                    let at = iter.position();
                    iter.next();
                    tokens.push(Token::Semicolon(at));
                };
            } else if c == '|' {
                if let Some('|') = iter.d { // Repeated escape sequence
                    let token = lex_word(&mut iter)?;
                    tokens.push(token);
                } else { // Bar
                    let at = iter.position();
                    iter.next();
                    tokens.push(Token::Bar(at));
                };
            } else if c == '~' {
                if let Some('~') = iter.d { // Repeated escape sequence
                    let token = lex_word(&mut iter)?;
                    tokens.push(token);
                } else { // Tilde
                    let at = iter.position();
                    iter.next();
                    tokens.push(Token::Tilde(at));
                }
            } else if c == '#' {
                if let Some(d) = iter.d {
                    if d == '#' || d.is_whitespace() { // Comment whitespace
                        let whitespace = lex_whitespace(&mut iter)?;
                        tokens.push(whitespace);
                    } else if d == ':' || d == ';' || d == '|' || d == '~' || d == '<' || d == '>' { // Maybe repeated escape sequence
                        let at = iter.position();
                        return Err(LexError::InvalidHashSequence(at));
                        // TODO: Allow repeated escape sequence. Maybe need 3 lookahead
                    } else if d == '{' || d == '}' || d == '[' || d == ']' || d == '"'  { // Illegal
                        let at = iter.position();
                        return Err(LexError::InvalidHashSequence(at));
                    } else { // Hash text glyph
                        let word = lex_word(&mut iter)?;
                        tokens.push(word);
                    };
                } else { // Comment before end
                    let whitespace = lex_whitespace(&mut iter)?;
                    tokens.push(whitespace);
                };
            } else if c.is_whitespace() { // Whitespace
                let whitespace = lex_whitespace(&mut iter)?;
                tokens.push(whitespace);
            } else if c == '`' { // Character escape sequence
                let word = lex_word(&mut iter)?;
                tokens.push(word);
            } else { // Text glyph
                let word = lex_word(&mut iter)?;
                tokens.push(word);
            };
        } else {
            tokens.push(Token::End(iter.position()));
            break;
        };
    };
    Ok(tokens)
}

/// Lex whitespace.
///
/// Assumes that the current character is whitespace or a hash opening a comment.
fn lex_whitespace<It: Iterator<Item = char>>(iter: &mut CharIter<It>) -> Result<Token, LexError> {
    let at = iter.position();
    loop {
        if let Some(c) = iter.c {
            if c.is_whitespace() {
                iter.next();
            } else if c == '#' {
                if let Some(d) = iter.d {
                    if d.is_whitespace() || d == '#' {
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
            if c == '{' || c == '}' || c == '[' || c == ']' || c == '"' { // Reserved
                break;
            } else if c == '<' || c == '>' || c == ':' || c == '|' || c == ';' || c == '~' { // Maybe escape sequence, otherwise reserved
                if let Some(d) = iter.d {
                    if d == c {
                        iter.next_two();
                        string.push(c);
                        string.push(c);
                        loop {
                            if let Some(e) = iter.c {
                                if c == e {
                                    iter.next();
                                    string.push(c);
                                } else {
                                    break;
                                };
                            } else {
                                break;
                            };
                        };
                    } else {
                        break;
                    };
                } else {
                    break;
                };
            } else if c == '`' { // Character escape character
                if let Some(d) = iter.d {
                    let e = match translate_escape_character(d) {
                        Ok(e) => e,
                        Err(..) => return Err(LexError::InvalidEscapeSequence(iter.position())),
                    };
                    iter.next_two();
                    string.push(e);
                } else {
                    return Err(LexError::EscapeEOS);
                };
            } else if c == '#' {
                if let Some(d) = iter.d {
                    if d == '#' || d.is_whitespace() || d == '{' || d == '}' || d == '[' || d == ']' || d == '<' || d == '>' || d == '"' || d == ':' || d == ';' || d == '|' || d == '~' { // Comment, repeated escape sequence or illegal
                        break;
                    } else { // # Hash text glyph
                        iter.next();
                        string.push('#');
                    };
                } else {
                    break;
                };
            } else if c.is_whitespace() { // Whitespace
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

/// Lex a quotation.
///
/// Assumes that the current character is `"`.
fn lex_quotation<It: Iterator<Item = char>>(iter: &mut CharIter<It>) -> Result<Token, LexError> {
    let at = iter.position();
    let mut string = String::new();
    iter.next();
    loop {
        if let Some(c) = iter.c {
            if c == '"' {
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
                    return Err(LexError::EscapeEOS);
                };
            } else if c == '\n' {
                return Err(LexError::UnclosedQuotation(iter.position()));
            } else {
                iter.next();
                string.push(c);
            };
        } else {
            return Err(LexError::UnclosedQuotation(iter.position()));
        };
    };
    iter.next();
    Ok(Token::Quotation(at, string))
}

/// Lex a text block.
///
/// Assumes that the current characters are `<#`.
fn lex_text_block<It: Iterator<Item = char>>(iter: &mut CharIter<It>, at: Position) -> Result<Token, LexError> {
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
            } else if c.is_whitespace() {
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
                        } else if c.is_whitespace() {
                            break;
                        } else {
                            return Err(LexError::InvalidTextBlockConfiguration(iter.position()))
                        }
                    } else {
                        return Err(LexError::UnclosedTextBlockTag(iter.position()));
                    }
                }
                skip_whitespace_in_text_block_tag(iter, at)?;
                if let Some(c) = iter.c {
                    if c == '>' {
                        iter.next();
                        break 'tag;
                    } else {
                        return Err(LexError::UnclosedTextBlockTag(iter.position()));
                    }
                }
            } else {
                iter.next();
                closing_tag.push(c);
            }
        } else {
            return Err(LexError::UnclosedTextBlockTag(iter.position()));
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
            return Err(LexError::UnclosedQuotation(iter.position()));
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
            if !c.is_whitespace() {
                return Ok(());
            }
        } else {
            return Err(LexError::UnclosedTextBlockTag(at));
        }
    }
}

pub enum LexError {
    /// Tried to escape EOS.
    EscapeEOS,
    /// Quotation was not closed before the end of the line.
    /// Or, text block was not closed before EOS.
    UnclosedQuotation(Position),
    /// Character escape sequence is not recognized.
    InvalidEscapeSequence(Position),
    /// Illegal character after hash.
    InvalidHashSequence(Position),
    UnclosedTextBlockTag(Position),
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
