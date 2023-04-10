use std::str::Chars;
use crate::lex::Token::Whitespace;


/// A char position.
///
/// Contains a line number and a column number, corresponding to a character in a document.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Position { pub index: usize, pub line: usize, pub column: usize }


#[derive(PartialEq, Eq, Clone)]
pub enum Token {
    BracketOpening(Position),
    BracketClosing(Position),
    Semicolon(Position),
    Colon(Position),
    Comment(Position),
    SequenceOpening(Position),
    SequenceClosing(Position),
    Word(Position, String),
    Quote(Position, String),
    CommandOpening(Position),
    CommandClosing(Position),
    Whitespace(Position),
    End(Position),
}


pub struct CharIter<'a> {
    chars: Chars<'a>,
    c: Option<char>,
    cn: Option<char>,
    index: usize,
    line: usize,
    column: usize,
    word: Option<(String, Position)>,
}


impl <'a> CharIter<'a> {

    pub fn new(mut chars: Chars<'a>) -> Self {
        let c = chars.next();
        let cn = chars.next();
        CharIter {
            chars,
            c,
            cn,
            index: 0,
            line: 1,
            column: 1,
            word: None
        }
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
        self.c = self.cn;
        self.cn = self.chars.next();
    }

    pub fn position(&self) -> Position {
        Position {
            index: self.index,
            line: self.line,
            column: self.column,
        }
    }

    fn put_char(&mut self, char: char) {
        if let Some(word) = self.word.as_mut() {
            let (word, ..) = word;
            word.push(char);
        } else {
            self.word = Some((String::from(char), self.position()));
        };
    }

    fn put_char_at(&mut self, char: char, at: Position) {
        if let Some(word) = self.word.as_mut() {
            let (word, ..) = word;
            word.push(char);
        } else {
            self.word = Some((String::from(char), at));
        };
    }

    fn flush_word(&mut self, tokens: &mut Vec<Token>) {
        if let Some((word, position)) = self.word.take() {
            tokens.push(Token::Word(position, word));
        };
    }

    fn flush_quote(&mut self, tokens: &mut Vec<Token>) {
        if let Some((word, position)) = self.word.take() {
            tokens.push(Token::Quote(position, word));
        };
    }

    fn skip_whitespace(&mut self) {
        loop {
            if let Some(c) = self.c {
                if c.is_whitespace() {
                    self.next();
                } else {
                    break;
                }
            } else {
                break;
            }
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


pub enum LexError {
    EscapeEOS,
    CommentedBracket(Position),
    UnclosedQuote(Position),
}


pub struct LexedStr {
    tokens: Vec<Token>,
    procstr: String,
}


impl <'a> CharIter<'a> {

    pub fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = vec![];
        loop {
            if let Some(c) = self.c {
                if c == '{' {
                    self.flush_word(&mut tokens);
                    tokens.push(Token::BracketOpening(self.position()));
                    self.next();
                } else if c == '}' {
                    self.flush_word(&mut tokens);
                    tokens.push(Token::BracketClosing(self.position()));
                    self.next();
                } else if c == '[' {
                    self.flush_word(&mut tokens);
                    tokens.push(Token::SequenceOpening(self.position()));
                    self.next();
                } else if c == ']' {
                    self.flush_word(&mut tokens);
                    tokens.push(Token::SequenceClosing(self.position()));
                    self.next();
                } else if c == '(' {
                    if let Some('(') = self.cn {
                        self.put_char('(');
                        loop {
                            self.next();
                            if let Some('(') = self.cn {
                                self.put_char('(');
                            } else {
                                break;
                            };
                        };
                    } else {
                        self.flush_word(&mut tokens);
                        tokens.push(Token::CommandOpening(self.position()));
                        self.next();
                    };
                } else if c == ')' {
                    if let Some(')') = self.cn {
                        self.put_char(')');
                        loop {
                            self.next();
                            if let Some(')') = self.cn {
                                self.put_char(')');
                            } else {
                                break;
                            };
                        };
                    } else {
                        self.flush_word(&mut tokens);
                        tokens.push(Token::CommandClosing(self.position()));
                        self.next();
                    };
                } else if c == ';' {
                    self.flush_word(&mut tokens);
                    tokens.push(Token::Semicolon(self.position()));
                    self.next();
                } else if c == ':' {
                    if let Some(':') = self.cn {
                        self.put_char(':');
                        loop {
                            self.next();
                            if let Some(':') = self.cn {
                                self.put_char(':');
                            } else {
                                break;
                            };
                        };
                    } else {
                        self.flush_word(&mut tokens);
                        tokens.push(Token::Colon(self.position()));
                        self.next();
                    };
                } else if c == '#' {
                    if let &Some((..)) = &self.word {
                        self.put_char('#');
                        self.next();
                    } else {
                        if let Some(cn) = self.cn {
                            if cn.is_whitespace() || cn == '#' || cn == ';' || cn == ':' || cn == '}' || cn == ']' || cn == ')' {
                                self.flush_word(&mut tokens);
                                tokens.push(Token::Comment(self.position()));
                                self.skip_line();
                            } else if cn == '{' || cn == '[' || cn == '(' {
                                return Err(LexError::CommentedBracket(self.position()));
                            } else {
                                self.put_char('#');
                                self.next();
                            };
                        } else {
                            self.flush_word(&mut tokens);
                            tokens.push(Token::Comment(self.position()));
                            self.next();
                        };
                    };
                } else if c == '\\' {
                    if let Some(cn) = self.cn {
                        self.put_char(cn);
                        self.next();
                        self.next();
                    } else {
                        return Err(LexError::EscapeEOS);
                    };
                } else if c == '"' {
                    self.flush_word(&mut tokens);
                    self.next();
                    loop {
                        if let Some(c) = self.c {
                            if c == '"' {
                                break;
                            } else if c == '\\' {
                                if let Some(cn) = self.cn {
                                    self.put_char(cn);
                                    self.next(); self.next();
                                } else {
                                    return Err(LexError::EscapeEOS);
                                };
                            } else {
                                self.put_char(c);
                            };
                            self.next();
                        } else {
                            return Err(LexError::UnclosedQuote(self.position()))
                        };
                    };
                    self.flush_quote(&mut tokens);
                    self.next();
                } else if c.is_whitespace() {
                    self.flush_word(&mut tokens);
                    tokens.push(Whitespace(self.position()));
                    self.skip_whitespace();
                } else {
                    self.put_char(c);
                    self.next();
                };
            } else {
                tokens.push(Token::End(self.position()));
                return Ok(tokens);
            };
        };
    }

}
