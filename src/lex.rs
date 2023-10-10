use crate::translate_escape_character;

//// Position

/// A char position.
///
/// Contains a line number and a column number, corresponding to a character in a document.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Position { pub index: usize, pub line: usize, pub column: usize }

//// Token

#[derive(PartialEq, Eq, Clone)]
pub enum Token {
    /// A Word token.
    Word(Position, String),
    Quote(Position, String),
    Colon(Position),
    Semicolon(Position),
    /// A `|` token.
    Bar(Position),
    Diamond(Position),
    LeftBracket(Position),
    RightBracket(Position),
    HashRightBracket(Position),
    LeftSquare(Position),
    RightSquare(Position),
    HashRightSquare(Position),
    LeftAngle(Position),
    RightAngle(Position),
    /// A sequence of whitespace.
    Whitespace(Position),
    End(Position),
}

impl Token {

    pub fn position(&self) -> Position {
        match self {
            Token::Word(at, ..) => *at,
            Token::Quote(at, ..) => *at,
            Token::Colon(at) => *at,
            Token::Semicolon(at) => *at,
            Token::Bar(at) => *at,
            Token::Diamond(at) => *at,
            Token::LeftBracket(at) => *at,
            Token::RightBracket(at) => *at,
            Token::HashRightBracket(at) => *at,
            Token::LeftSquare(at) => *at,
            Token::RightSquare(at) => *at,
            Token::HashRightSquare(at) => *at,
            Token::LeftAngle(at) => *at,
            Token::RightAngle(at) => *at,
            Token::Whitespace(at) => *at,
            Token::End(at) => *at,
        }
    }

}

//// Char iterator

/// A char iterator.
pub struct CharIter<It: Iterator<Item = char>> {
    chars: It,
    /// Current character
    c: Option<char>,
    /// Current + 1 character
    d: Option<char>,
    index: usize,
    line: usize,
    column: usize,
}

impl <'a, It: Iterator<Item = char>> CharIter<It> {

    pub fn new(mut chars: It) -> Self {
        let c = chars.next();
        let d = chars.next();
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
        self.d = self.chars.next();
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

pub enum LexError {
    EscapeEOS,
    CommentedBracket(Position),
    UnclosedQuote(Position),
    UnknownEscapeSequence(Position),
    InvalidHashSequence(Position),
}

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
                    if d == '<' {
                        // Text escape sequence <
                        let token = lex_word(&mut iter)?;
                        tokens.push(token);
                    } else if d == '>' { // Diamond <>
                        let at = iter.position();
                        iter.next_two();
                        tokens.push(Token::Diamond(at));
                    } else if d == '#' {
                        let at = iter.position();
                        iter.next_two();
                        if let Some('>') = iter.c { // todo: allow labels
                            iter.next();
                            let quote = lex_multiline_quote(&mut iter, at)?;
                            tokens.push(quote);
                        } else {
                            return Err(LexError::InvalidHashSequence(at));
                        };
                    } else {
                        let at = iter.position();
                        iter.next();
                        tokens.push(Token::LeftAngle(at));
                    };
                } else {
                    let at = iter.position();
                    iter.next();
                    tokens.push(Token::LeftAngle(at));
                };
            } else if c == '>' {
                if let Some('>') = iter.d {
                    let token = lex_word(&mut iter)?;
                    tokens.push(token);
                } else {
                    let at = iter.position();
                    iter.next();
                    tokens.push(Token::RightAngle(at));
                };
            } else if c == '"' {
                let token = lex_quote(&mut iter)?;
                tokens.push(token);
            } else if c == ':' {
                if let Some(':') = iter.d {
                    let token = lex_word(&mut iter)?;
                    tokens.push(token);
                } else {
                    let at = iter.position();
                    iter.next();
                    tokens.push(Token::Colon(at));
                };
            } else if c == ';' {
                if let Some(';') = iter.d {
                    let token = lex_word(&mut iter)?;
                    tokens.push(token);
                } else {
                    let at = iter.position();
                    iter.next();
                    tokens.push(Token::Semicolon(at));
                };
            } else if c == '|' {
                if let Some('|') = iter.d {
                    let token = lex_word(&mut iter)?;
                    tokens.push(token);
                } else {
                    let at = iter.position();
                    iter.next();
                    tokens.push(Token::Bar(at));
                };
            } else if c == '#' {
                if let Some(d) = iter.d {
                    if d == '#' || d.is_whitespace() { // Comment whitespace
                        let whitespace = lex_whitespace(&mut iter)?;
                        tokens.push(whitespace);
                    } else if d == '}' { // `#}`
                        let at = iter.position();
                        iter.next_two();
                        tokens.push(Token::HashRightBracket(at));
                    } else if d == ']' { // `#]`
                        let at = iter.position();
                        iter.next_two();
                        tokens.push(Token::HashRightSquare(at));
                    } else if d == '{' || d == '[' || d == '<' || d == '>' || d == '"' || d == ':' || d == ';' || d == '|'  { // Illegal
                        let at = iter.position();
                        return Err(LexError::InvalidHashSequence(at));
                    } else { // `#<c>` or `` #` ``
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
            } else if c == '`' { // Escape character
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

/// Lex a whitespace sequence.
///
/// Assumes that the current character is whitespace or a hash glyph opening a comment.
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
                } else {
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
/// Assumes that the current character is text.
fn lex_word<It: Iterator<Item = char>>(iter: &mut CharIter<It>) -> Result<Token, LexError> {
    let at = iter.position();
    let mut string = String::new();
    loop {
        if let Some(c) = iter.c {
            if c == '{' || c == '}' || c == '[' || c == ']' || c == '"' { // Reserved
                break;
            } else if c == '<' || c == '>' || c == ':' || c == ';' || c == '|' { // Maybe escape sequence, otherwise reserved
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
            } else if c == '`' { // Escape character
                if let Some(d) = iter.d {
                    let e = match translate_escape_character(d) {
                        Ok(e) => e,
                        Err(..) => return Err(LexError::UnknownEscapeSequence(iter.position())),
                    };
                    iter.next_two();
                    string.push(e);
                } else {
                    return Err(LexError::EscapeEOS);
                };
            } else if c == '#' {
                if let Some(d) = iter.d {
                    if d == '#' || d.is_whitespace() || d == '{' || d == '}' || d == '[' || d == ']' || d == '<' || d == '>' || d == '"' || d == ':' || d == ';' || d == '|' { // Comment, `#]`, `#}`, `#?` token or disallowed sequence.
                        break;
                    } else { // # before glyph
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

/// Lex quoted text.
///
/// Assumes that the current character is at the opening quote.
fn lex_quote<It: Iterator<Item = char>>(iter: &mut CharIter<It>) -> Result<Token, LexError> {
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
                        Err(..) => return Err(LexError::UnknownEscapeSequence(iter.position())),
                    };
                    iter.next_two();
                    string.push(e);
                } else {
                    return Err(LexError::EscapeEOS);
                };
            } else {
                iter.next();
                string.push(c);
            };
        } else {
            return Err(LexError::UnclosedQuote(iter.position()));
        };
    };
    iter.next();
    Ok(Token::Quote(at, string))
}

/// Lex a multiline quote. // todo: Fix spaghetti
fn lex_multiline_quote<It: Iterator<Item = char>>(iter: &mut CharIter<It>, at: Position) -> Result<Token, LexError> {
    loop { // Ignore the line after the quote opening, unless there are glyphs there.
        if let Some(c) = iter.c {
            if c == '\n' {
                iter.next();
                break;
            } else if c.is_whitespace() {
                iter.next();
            } else {
                break;
            };
        } else {
            return Err(LexError::UnclosedQuote(iter.position()));
        };
    };
    let mut read = true;
    let mut lines = vec![];
    let mut least_indentation = usize::MAX;
    'a: while read {
        loop { // Skip whitespace at start of line.
            if let Some(c) = iter.c {
                if c == '\n' {
                    iter.next();
                    lines.push((String::new(), 0));
                    continue 'a;
                } else if c.is_whitespace() {
                    iter.next();
                } else {
                    break;
                };
            } else {
                return Err(LexError::UnclosedQuote(iter.position()));
            };
        };
        let indentation = iter.column;
        let mut line = String::new();
        loop {
            if let Some(c) = iter.c {
                if c == '\n' {
                    iter.next();
                    break;
                } else if c == '<' {
                    iter.next();
                    if let Some(c) = iter.c {
                        if c == '#' {
                            iter.next();
                            if let Some(c) = iter.c {
                                if c == '>' {
                                    iter.next();
                                    read = false;
                                    break;
                                } else {
                                    line.push('<');
                                    line.push('#');
                                };
                            } else {
                                return Err(LexError::UnclosedQuote(iter.position()));
                            };
                        } else {
                            line.push('<');
                        };
                    } else {
                        return Err(LexError::UnclosedQuote(iter.position()));
                    };
                } else {
                    iter.next();
                    line.push(c);
                };
            } else {
                return Err(LexError::UnclosedQuote(iter.position()));
            };
        };
        if line.is_empty() {
            if read {
                lines.push((line, 0));
            }
        } else {
            if indentation < least_indentation {
                least_indentation = indentation;
            };
            lines.push((line, indentation));
        };
    };
    let mut quote = String::new();
    for (l, i) in lines {
        if i == 0 { // Empty line.
            quote.push('\n');
            continue;
        } else {
            for _ in 0..(i - least_indentation) {
                quote.push(' ');
            };
            quote.push_str(&l);
            quote.push('\n');
        };
    };
    Ok(Token::Quote(at, quote))
}
