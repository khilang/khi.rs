//! Parsing of UDL documents.
//!
//! The root element of a document can be an expression, a sequence or a dictionary. Use the
//! corresponding function to parse a document: either [parse_expression_document],
//! [parse_sequence_document] or [parse_dictionary_document].


use std::fmt::{Debug, Display, Formatter};
use std::slice::Iter;
use crate::ast::{ParsedCompound, ParsedDictionary, ParsedEntry, ParsedExpression, ParsedSequence, ParsedText, ParsedAttribute, ParsedDirective};
use crate::lex::{CharIter, LexError, Position, Token};


/// Parse an expression document.
pub fn parse_expression_document(document: &str) -> Result<ParsedExpression, ParseError> {
    let tokens = tokenize(document)?;
    let mut iter = TokenIter::new(tokens.iter());
    let expression = iter.parse_expression()?;
    if !matches!(iter.t, Token::End(..)) {
        return Err(ParseError::ExpectedEnd(iter.position()));
    };
    Ok(expression)
}


/// Parse a sequence document.
pub fn parse_sequence_document(document: &str) -> Result<ParsedSequence, ParseError> {
    let tokens = tokenize(document)?;
    let mut iter = TokenIter::new(tokens.iter());
    let sequence = iter.parse_sequence()?;
    if !matches!(iter.t, Token::End(..)) {
        return Err(ParseError::ExpectedEnd(iter.position()));
    };
    Ok(sequence)
}


/// Parse a dictionary document.
pub fn parse_dictionary_document(document: &str) -> Result<ParsedDictionary, ParseError> {
    let tokens = tokenize(document)?;
    let mut iter = TokenIter::new(tokens.iter());
    let dictionary = iter.parse_dictionary()?;
    if !matches!(iter.t, Token::End(..) ) {
        return Err(ParseError::ExpectedEnd(iter.position()));
    };
    Ok(dictionary)
}


fn tokenize(document: &str) -> Result<Vec<Token>, ParseError> {
    let chars = document.chars();
    let mut iter = CharIter::new(chars);
    let tokens = match iter.lex() {
        Ok(ok) => ok,
        Err(err) => {
            return match err {
                LexError::EscapeEOS => Err(ParseError::EscapingEndOfStream),
                LexError::CommentedBracket(at) => Err(ParseError::CommentedBracket(at)),
                LexError::UnclosedQuote(at) => Err(ParseError::UnclosedQuote(at)),
            };
        }
    };
    Ok(tokens)
}


struct TokenIter<'a> {
    tokens: Iter<'a, Token>,
    t: &'a Token,
    t2: &'a Token,
}


impl <'a> TokenIter<'a> {

    pub fn new(mut tokens: Iter<'a, Token>) -> Self {
        let t = tokens.next().unwrap();
        let t2 = tokens.next().unwrap_or(t);
        TokenIter { tokens, t, t2 }
    }

    pub fn next(&mut self) {
        self.t = self.t2;
        self.t2 = self.tokens.next().unwrap_or(self.t);
    }

    fn skip_whitespace(&mut self) {
        loop {
            if matches!(self.t, Token::Whitespace(..) | Token::Comment(..)) {
                self.next();
            } else {
                break;
            };
        };
    }

    fn skip_lookahead_whitespace(&mut self) {
        loop {
            if matches!(self.t2, Token::Whitespace(..) | Token::Comment(..)) {
                self.t2 = self.tokens.next().unwrap_or(self.t2);
            } else {
                break;
            };
        };
    }

    pub fn is_whitespace(&self) -> bool {
        match self.t {
            Token::Whitespace(..) | Token::Comment(..) => true,
            _ => false,
        }
    }

    pub fn position(&self) -> Position {
        match self.t {
            Token::BracketOpening(position) => position,
            Token::BracketClosing(position) => position,
            Token::Semicolon(position) => position,
            Token::Colon(position) => position,
            Token::Comment(position) => position,
            Token::Diamond(position) => position,
            Token::OpeningTagOpening(position) => position,
            Token::SequenceOpening(position) => position,
            Token::SequenceClosing(position) => position,
            Token::Word(position, ..) => position,
            Token::Quote(position, ..) => position,
            Token::DirectiveOpening(position) => position,
            Token::ClosingTagOpening(position) => position,
            Token::DirectiveClosing(position) => position,
            Token::Whitespace(position) => position,
            Token::End(position) => position,
        }.clone()
    }

    pub fn next_position(&self) -> Position {
        match self.t2 {
            Token::BracketOpening(position) => position,
            Token::BracketClosing(position) => position,
            Token::Semicolon(position) => position,
            Token::Colon(position) => position,
            Token::Comment(position) => position,
            Token::Diamond(position) => position,
            Token::OpeningTagOpening(position) => position,
            Token::SequenceOpening(position) => position,
            Token::SequenceClosing(position) => position,
            Token::Word(position, ..) => position,
            Token::Quote(position, ..) => position,
            Token::DirectiveOpening(position) => position,
            Token::ClosingTagOpening(position) => position,
            Token::DirectiveClosing(position) => position,
            Token::Whitespace(position) => position,
            Token::End(position) => position,
        }.clone()
    }

}


impl <'a> TokenIter<'a> {

    /// Parse an expression.
    ///
    /// ```text
    /// <expr> ::= <exprp>
    ///          | <>
    ///
    /// <exprp> ::= <text> <expr>
    ///           | "{" <expr> "}" <expr>
    ///           | "{" <dictp> "}" <expr>
    ///           | "[" <seq> "]" <expr>
    ///           | <direxpr> <expr>
    /// ```
    pub fn parse_expression(&mut self) -> Result<ParsedExpression, ParseError> {
        let mut arguments = vec![];
        let from = self.position();
        loop {
            self.skip_whitespace();
            match self.t {
                Token::Word(_, w) => {
                    let mut text = String::new();
                    let mut whitespace = false;
                    text.push_str(w);
                    let mut to = self.next_position();
                    loop {
                        self.next();
                        match self.t {
                            Token::Comment(..) | Token::Whitespace(..) => {
                                whitespace = true;
                            }
                            Token::Word(_, w) => {
                                text.push(' ');
                                text.push_str(w);
                                to = self.next_position();
                                whitespace = false;
                            }
                            Token::BracketOpening(..) | Token::BracketClosing(..) | Token::Semicolon(..) | Token::Colon(..) | Token::Diamond(..) | Token::OpeningTagOpening(..) | Token::SequenceOpening(..) | Token::SequenceClosing(..) | Token::Quote(..) | Token::DirectiveOpening(..) | Token::ClosingTagOpening(..) | Token::DirectiveClosing(..) | Token::End(..) => {
                                break;
                            }
                        };
                    };
                    arguments.push((ParsedText { text, from, to }.into(), whitespace));
                }
                Token::Quote(quote_from, q) => {
                    self.next();
                    let quote_to = self.position();
                    let whitespace = self.is_whitespace();
                    arguments.push((ParsedText { text: String::from(q), from: quote_from.clone(), to: quote_to }.into(), whitespace));

                }
                Token::BracketOpening(..) => {
                    let argument = self.parse_bracket()?;
                    let whitespace = self.is_whitespace();
                    arguments.push((argument, whitespace));
                }
                Token::SequenceOpening(..) => {
                    self.next();
                    let sequence = self.parse_sequence()?;
                    if !matches!(self.t, Token::SequenceClosing(..)) {
                        return Err(ParseError::ExpectedSequenceClosing(self.position()));
                    };
                    self.next();
                    let whitespace = self.is_whitespace();
                    arguments.push((sequence.into(), whitespace));
                }
                Token::DirectiveOpening(..) | Token::OpeningTagOpening(..) => {
                    let directive = self.parse_directive_expression()?;
                    let whitespace = self.is_whitespace();
                    arguments.push((directive.into(), whitespace));
                }
                Token::BracketClosing(to) | Token::SequenceClosing(to) | Token::Diamond(to) | Token::DirectiveClosing(to) | Token::Semicolon(to) | Token::Colon(to) | Token::End(to) | Token::ClosingTagOpening(to) => {
                    return if arguments.len() == 0 {
                        Ok(ParsedExpression::Empty(from, to.clone()))
                    } else if arguments.len() == 1 {
                        let (argument, ..) = arguments.pop().unwrap();
                        Ok(argument.into())
                    } else {
                        Ok(ParsedCompound { arguments, from, to: to.clone() }.into())
                    };
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
        }
    }

    /// Parse a sequence.
    ///
    /// ```text
    /// <seq> ::= <exprp>
    ///         | <expr> ";" <seq>
    ///         | ":" <seq>
    ///         | <>
    /// ```
    pub fn parse_sequence(&mut self) -> Result<ParsedSequence, ParseError> {
        let mut elements = vec![];
        let from = self.position();
        loop {
            let element_from = self.position();
            self.skip_whitespace();
            match self.t {
                Token::BracketOpening(..) | Token::SequenceOpening(..) | Token::DirectiveOpening(..) | Token::OpeningTagOpening(..) | Token::Word(..) | Token::Quote(..) => {
                    let expression = self.parse_expression()?;
                    elements.push(expression);
                    self.skip_whitespace();
                    if let Token::Semicolon(..) = self.t {
                        self.next();
                    } else {
                        let to = self.position();
                        return Ok(ParsedSequence { elements, from, to });
                    };
                }
                Token::Semicolon(element_to) => {
                    elements.push(ParsedExpression::Empty(element_from, element_to.clone()));
                    self.next();
                }
                Token::BracketClosing(position) | Token::SequenceClosing(position) | Token::DirectiveClosing(position) | Token::Diamond(position) | Token::ClosingTagOpening(position) | Token::End(position) => {
                    return Ok(ParsedSequence { elements, from, to: position.clone() });
                }
                Token::Colon(..) => {
                    self.next();
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
        }
    }

    /// Parse a dictionary.
    ///
    /// ```text
    /// <dict> ::= <dictp>
    ///          | <>
    ///
    /// <dictp> ::= <key> ":" <expr>
    ///           | <key> ":" <expr> ";" <dict>
    ///           | <key> ";" <dict>
    ///           | ":" <dict>
    /// ```
    pub fn parse_dictionary(&mut self) -> Result<ParsedDictionary, ParseError> {
        let from = self.position();
        let mut entries = vec![];
        loop {
            self.skip_whitespace();
            let key = match self.t {
                Token::Word(key_from, w) | Token::Quote(key_from, w) => {
                    let key_to = self.next_position();
                    ParsedText {
                        text: String::from(w),
                        from: key_from.clone(),
                        to: key_to,
                    }
                }
                Token::Colon(..) => {
                    self.next();
                    continue;
                },
                Token::BracketOpening(to) | Token::DirectiveOpening(to) | Token::Diamond(to) | Token::OpeningTagOpening(to) | Token::ClosingTagOpening(to) | Token::DirectiveClosing(to) | Token::BracketClosing(to) | Token::End(to) | Token::Semicolon(to) | Token::SequenceOpening(to) | Token::SequenceClosing(to) => {
                    return Ok(ParsedDictionary { entries, from, to: to.clone() });
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
            self.next(); self.skip_whitespace();
            // Look for assignment colon, entry separator or end.
            match self.t {
                Token::Colon(..) => { }
                Token::Semicolon(entry_at) => {
                    entries.push(ParsedEntry {
                        key,
                        value: ParsedExpression::Empty(entry_at.clone(), entry_at.clone()),
                    });
                    self.next();
                    continue;
                }
                Token::BracketOpening(at) | Token::BracketClosing(at) | Token::SequenceOpening(at) | Token::Diamond(at) | Token::OpeningTagOpening(at) | Token::SequenceClosing(at) | Token::Word(at, ..) | Token::Quote(at, ..) | Token::DirectiveOpening(at) | Token::ClosingTagOpening(at) | Token::DirectiveClosing(at) | Token::End(at) => {
                    return Err(ParseError::ExpectedEntrySeparator(at.clone()));
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
            self.next();
            // After colon. Take expression value.
            let expression = self.parse_expression()?;
            entries.push(ParsedEntry {
                key,
                value: expression.into(),
            });
            // Look for entry separator or end.
            match self.t {
                Token::Semicolon(..) => {
                    self.next();
                    continue;
                }
                Token::Word(to, ..) | Token::Quote(to, ..) | Token::BracketOpening(to) | Token::Diamond(to) | Token::OpeningTagOpening(to) | Token::SequenceOpening(to) | Token::DirectiveOpening(to) | Token::ClosingTagOpening(to) | Token::End(to) | Token::BracketClosing(to) | Token::SequenceClosing(to) | Token::DirectiveClosing(to) | Token::Colon(to) => {
                    return Ok(ParsedDictionary { entries, from, to: to.clone() });
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
        };
    }


    /// Parse a directive expression.
    ///
    /// ```text
    /// <direxpr> ::= <><cmdexpr>
    ///             | <><tagexpr>
    ///
    /// <attr> ::= <>
    ///          | <key> <attr>
    ///          | <key> ":" <key> <attr>
    ///          | <key> ":" "{" <expr> "}" <attr>
    ///          | <key> ":" "{" <dictp> "}" <attr>
    ///          | <key> ":" "[" <seq> "]" <attr>
    /// ```
    pub fn parse_directive_expression(&mut self) -> Result<ParsedDirective, ParseError> {
        if matches!(self.t, Token::DirectiveOpening(..)) {
            self.parse_command_expression()
        } else if matches!(self.t, Token::OpeningTagOpening(..)) {
            self.parse_tag_expression()
        } else {
            Err(ParseError::ExpectedOpeningAngularBracket(self.position()))
        }
    }


    /// Parse a command expression.
    ///
    /// ```text
    /// <cmdexpr> ::= <>"<" <key> <attr> ">"<cmdarg>
    ///
    /// <cmdarg> ::= < >
    ///            | <>":"<key><cmdarg>
    ///            | <>":""{" <expr> "}"<cmdarg>
    ///            | <>":""{" <dictp> "}"<cmdarg>
    ///            | <>":""[" <seq> "]"<cmdarg>
    ///            | <>":""<" <key> <attr> ">"<cmdarg>
    ///            | <>":""<>"":"<cmdexpr>
    /// ```
    pub fn parse_command_expression(&mut self) -> Result<ParsedDirective, ParseError> {
        let from = self.position();
        if !matches!(self.t, Token::DirectiveOpening(..)) {
            return Err(ParseError::ExpectedOpeningAngularBracket(from));
        }
        self.next(); self.skip_whitespace();
        let directive = match self.t {
            Token::Word(p, k) | Token::Quote(p, k) => k.clone(),
            _ => return Err(ParseError::ExpectedDirectiveKey(self.position())),
        };
        self.next(); self.skip_whitespace();
        let attributes = self.parse_attributes()?;
        if !matches!(self.t, Token::DirectiveClosing(..)) {
            return Err(ParseError::ExpectedDirectiveClosing(self.position()))
        }
        self.next();
        let mut arguments = vec![];
        loop {
            if !matches!(self.t, Token::Colon(..)) {
                let to = self.position();
                return Ok(ParsedDirective {
                    directive,
                    attributes,
                    arguments,
                    from,
                    to,
                });
            };
            self.next();
            match self.t {
                Token::Word(from, w) | Token::Quote(from, w) => {
                    let to = self.next_position();
                    let text = ParsedText {
                        text: String::from(w),
                        from: from.clone(),
                        to,
                    };
                    arguments.push(text.into());
                    self.next();
                }
                Token::BracketOpening(..) => {
                    let argument = self.parse_bracket()?;
                    arguments.push(argument.into());
                }
                Token::SequenceOpening(..) => {
                    self.next();
                    let sequence = self.parse_sequence()?;
                    arguments.push(sequence.into());
                    if !matches!(self.t, Token::SequenceClosing(..)) {
                        return Err(ParseError::ExpectedSequenceClosing(self.position()));
                    };
                    self.next();
                }
                Token::Diamond(..) => {
                    self.next();
                    if !matches!(self.t, Token::Colon(..)) {
                        let position = self.position();
                        return Err(ParseError::ExpectedColonAfterPrecedenceOperator(position));
                    };
                    self.next();
                    // Gives a more specific error message.
                    if !matches!(self.t, Token::DirectiveOpening(..)) {
                        let position = self.position();
                        return Err(ParseError::ExpectedDirectiveAfterPrecedenceOperator(position));
                    };
                    let right_hand_directive = self.parse_command_expression()?;
                    arguments.push(right_hand_directive.into());
                    let to = self.position();
                    return Ok(ParsedDirective {
                        directive,
                        attributes,
                        arguments,
                        from,
                        to,
                    });
                }
                Token::DirectiveOpening(..) => {
                    let directive_from = self.position();
                    self.next(); self.skip_whitespace();
                    let directive = match self.t {
                        Token::Word(_, w) | Token::Quote(_, w) => w.clone(),
                        Token::BracketOpening(_) | Token::BracketClosing(_) | Token::Semicolon(_) | Token::OpeningTagOpening(_) | Token::Diamond(_) | Token::DirectiveClosing(_) | Token::Colon(_) | Token::SequenceOpening(_) | Token::SequenceClosing(_) | Token::DirectiveOpening(_) | Token::ClosingTagOpening(_) | Token::End(_) => {
                            return Err(ParseError::ExpectedDirectiveKey(self.position()));
                        }
                        Token::Whitespace(_) | Token::Comment(_)  => unreachable!(),
                    };
                    self.next();
                    let attributes = self.parse_attributes()?;
                    if !matches!(self.t, Token::DirectiveClosing(..)) {
                        return Err(ParseError::ExpectedDirectiveClosing(self.position()));
                    };
                    let directive_to = self.position();
                    self.next();
                    arguments.push(ParsedDirective {
                        directive,
                        attributes,
                        arguments: vec![],
                        from: directive_from,
                        to: directive_to,
                    }.into());
                }
                Token::Whitespace(position) | Token::Comment(position) | Token::Colon(position) | Token::Semicolon(position) | Token::BracketClosing(position) | Token::SequenceClosing(position) | Token::DirectiveClosing(position) | Token::End(position) | Token::ClosingTagOpening(position) => {
                    return Err(ParseError::ExpectedDirectiveArgument(position.clone()));
                }
                Token::OpeningTagOpening(at) => {
                    return Err(ParseError::IllegalTagDirectiveArgument(at.clone()))
                }
            };
        };
    }


    /// Parse a tag expression.
    ///
    /// ```text
    /// <tagexpr> ::= "<+" <key> <attr> ">"<tagarg> <expr> "<-" <key> ">"
    ///             | "<+" <key> <attr> ">"<tagarg> <expr> "<-" ">"
    ///
    ///
    /// <tagarg> ::= < >
    ///            | ":"<key><tagarg>
    ///            | ":""{" <expr> "}"<tagarg>
    ///            | ":""{" <dictp> "}"<tagarg>
    ///            | ":""[" <seq> "]"<tagarg>
    ///            | ":""<" <key> <attr> ">"<tagarg>
    /// ```
    pub fn parse_tag_expression(&mut self) -> Result<ParsedDirective, ParseError> {
        let from = self.position();
        if !matches!(self.t, Token::OpeningTagOpening(..)) {
            return Err(ParseError::ExpectedOpeningAngularBracket(from));
        }
        self.next(); self.skip_whitespace();
        let directive = match self.t {
            Token::Word(p, k) | Token::Quote(p, k) => k.clone(),
            _ => return Err(ParseError::ExpectedDirectiveKey(self.position())),
        };
        self.next();
        let attributes = self.parse_attributes()?;
        if !matches!(self.t, Token::DirectiveClosing(..)) {
            return Err(ParseError::ExpectedDirectiveClosing(self.position()))
        }
        self.next();
        let mut arguments = vec![];
        loop {
            if !matches!(self.t, Token::Colon(..)) {
                break;
            };
            self.next();
            match self.t {
                Token::Word(from, w) | Token::Quote(from, w) => {
                    let to = self.next_position();
                    let text = ParsedText {
                        text: String::from(w),
                        from: from.clone(),
                        to,
                    };
                    arguments.push(text.into());
                    self.next();
                }
                Token::BracketOpening(..) => {
                    let argument = self.parse_bracket()?;
                    arguments.push(argument.into());
                }
                Token::SequenceOpening(..) => {
                    self.next();
                    let sequence = self.parse_sequence()?;
                    arguments.push(sequence.into());
                    if !matches!(self.t, Token::SequenceClosing(..)) {
                        return Err(ParseError::ExpectedSequenceClosing(self.position()));
                    };
                    self.next();
                }
                Token::DirectiveOpening(directive_from) => {
                    self.next(); self.skip_whitespace();
                    let directive = match self.t {
                        Token::Word(_, w) | Token::Quote(_, w) => w.clone(),
                        Token::BracketOpening(_) | Token::BracketClosing(_) | Token::Semicolon(_) | Token::OpeningTagOpening(_) | Token::Diamond(_) | Token::DirectiveClosing(_) | Token::Colon(_) | Token::SequenceOpening(_) | Token::SequenceClosing(_) | Token::DirectiveOpening(_) | Token::ClosingTagOpening(_) | Token::End(_) => {
                            return Err(ParseError::ExpectedDirectiveKey(self.position()));
                        }
                        Token::Whitespace(_) | Token::Comment(_)  => unreachable!(),
                    };
                    self.next();
                    let attributes = self.parse_attributes()?;
                    if !matches!(self.t, Token::DirectiveClosing(..)) {
                        return Err(ParseError::ExpectedDirectiveClosing(self.position()));
                    };
                    let directive_to = self.position();
                    self.next();
                    arguments.push(ParsedDirective {
                        directive,
                        attributes,
                        arguments: vec![],
                        from: directive_from.clone(),
                        to: directive_to,
                    }.into());
                }
                Token::Whitespace(position) | Token::Comment(position) | Token::Diamond(position) | Token::Colon(position) | Token::Semicolon(position) | Token::BracketClosing(position) | Token::SequenceClosing(position) | Token::DirectiveClosing(position) | Token::End(position) | Token::ClosingTagOpening(position) => {
                    return Err(ParseError::ExpectedDirectiveArgument(position.clone()));
                }
                Token::OpeningTagOpening(at) => {
                    return Err(ParseError::IllegalTagDirectiveArgument(at.clone()));
                }
            };
        };
        let content = self.parse_expression()?;
        arguments.push(content);
        if !matches!(self.t, Token::ClosingTagOpening(..)) {
            return Err(ParseError::ExpectedClosingTag(self.position(), directive));
        };
        let closing_tag_at = self.position();
        self.next(); self.skip_whitespace();
        match self.t {
            Token::DirectiveClosing(to) => {
                self.next();
                return Ok(ParsedDirective {
                    directive,
                    attributes,
                    arguments,
                    from,
                    to: to.clone(),
                });
            }
            Token::Word(at, w) | Token::Quote(at, w) => {
                if !directive.eq(w) {
                    return Err(ParseError::MismatchedClosingTag(from, directive, closing_tag_at, w.clone()));
                };
                self.next(); self.skip_whitespace();
                if !matches!(self.t, Token::DirectiveClosing(..)) {
                    return Err(ParseError::ExpectedDirectiveClosing(self.position()));
                };
                let to = self.position();
                self.next();
                return Ok(ParsedDirective {
                    directive,
                    attributes,
                    arguments,
                    from,
                    to,
                });
            }
            Token::End(at) | Token::BracketOpening(at) | Token::BracketClosing(at) | Token::Diamond(at) | Token::OpeningTagOpening(at) | Token::Semicolon(at) | Token::Colon(at) | Token::SequenceOpening(at) | Token::SequenceClosing(at) | Token::DirectiveOpening(at) | Token::ClosingTagOpening(at) => {
                return Err(ParseError::ExpectedDirectiveClosing(at.clone()));
            }
            Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
        };
    }


    /// Parse attributes.
    ///
    /// ```text
    /// <attr> ::= <>
    ///          | <key> <attr>
    ///          | <key> ":" <key> <attr>
    ///          | <key> ":" "{" <expr> "}" <attr>
    ///          | <key> ":" "{" <dictp> "}" <attr>
    ///          | <key> ":" "[" <seq> "]" <attr>
    ///          | <key> ":" <cmdexpr> <attr>
    /// ```
    pub fn parse_attributes(&mut self) -> Result<Vec<ParsedAttribute>, ParseError> {
        let mut attributes = vec![];
        self.skip_whitespace();
        loop {
            let key = match self.t {
                Token::Word(_, key) | Token::Quote(_, key) => {
                    let key_from = self.position();
                    let key_to = self.next_position();
                    ParsedText {
                        text: String::from(key),
                        from: key_from,
                        to: key_to,
                    }
                }
                Token::BracketOpening(..) | Token::BracketClosing(..) | Token::Diamond(..) | Token::OpeningTagOpening(..) | Token::Semicolon(..) | Token::Colon(..) | Token::SequenceOpening(..) | Token::SequenceClosing(..) | Token::DirectiveOpening(..) | Token::ClosingTagOpening(..) | Token::DirectiveClosing(..) | Token::End(..) => {
                    return Ok(attributes);
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
            self.next(); self.skip_whitespace();
            match self.t {
                Token::Colon(..) => { }
                Token::BracketOpening(attribute_at) | Token::Diamond(attribute_at) | Token::OpeningTagOpening(attribute_at) | Token::BracketClosing(attribute_at) | Token::Semicolon(attribute_at) | Token::SequenceOpening(attribute_at) | Token::SequenceClosing(attribute_at) | Token::DirectiveOpening(attribute_at) | Token::ClosingTagOpening(attribute_at) | Token::DirectiveClosing(attribute_at) | Token::End(attribute_at) | Token::Word(attribute_at, ..) | Token::Quote(attribute_at, ..) => {
                    attributes.push(ParsedAttribute {
                        key,
                        value: ParsedExpression::Empty(attribute_at.clone(), attribute_at.clone()),
                    });
                    continue;
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
            self.next(); self.skip_whitespace();
            let value = match self.t {
                Token::Word(text_from, w) | Token::Quote(text_from, w) => {
                    let text_to = self.next_position();
                    let text = ParsedText { text: String::from(w), from: text_from.clone(), to: text_to };
                    self.next();
                    text.into()
                }
                Token::BracketOpening(..) => {
                    let argument = self.parse_bracket()?;
                    argument
                }
                Token::SequenceOpening(..) => {
                    self.next();
                    let sequence = self.parse_sequence()?;
                    if !matches!(self.t, Token::SequenceClosing(..)) {
                        let at = self.position();
                        return Err(ParseError::ExpectedClosingSquare(at));
                    };
                    self.next();
                    sequence.into()
                }
                Token::DirectiveOpening(..) => {
                    let directive = self.parse_command_expression()?;
                    directive.into()
                }
                Token::BracketClosing(at) | Token::Semicolon(at) | Token::Diamond(at) | Token::Colon(at) | Token::SequenceClosing(at) | Token::DirectiveClosing(at) | Token::End(at) | Token::ClosingTagOpening(at) => {
                    return Err(ParseError::ExpectedAttributeArgument(at.clone()));
                }
                Token::OpeningTagOpening(at) => {
                    return Err(ParseError::IllegalTagAttribute(at.clone()));
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
            attributes.push(ParsedAttribute { key, value });
            self.skip_whitespace();
        };
    }

    /// Parse a bracket.
    ///
    /// Handles the patterns:
    /// ```text
    /// <>"{" <expr> "}"
    /// <>"{" <dictp> "}"
    /// ```
    fn parse_bracket(&mut self) -> Result<ParsedExpression, ParseError> {
        let argument;
        self.next(); self.skip_whitespace();
        match self.t {
            Token::SequenceOpening(..) | Token::BracketOpening(..) | Token::DirectiveOpening(..) | Token::OpeningTagOpening(..) | Token::ClosingTagOpening(..) => {
                let expression = self.parse_expression()?;
                argument = expression.into();
            }
            Token::Colon(..) => {
                let dictionary = self.parse_dictionary()?;
                argument = dictionary.into();
            }
            Token::Word(..) | Token::Quote(..) => {
                self.skip_lookahead_whitespace();
                match self.t2 {
                    Token::Semicolon(..) | Token::Colon(..) => {
                        let dictionary = self.parse_dictionary()?;
                        argument = dictionary.into();
                    }
                    Token::Word(..) | Token::Quote(..) | Token::Diamond(..) | Token::OpeningTagOpening(..) | Token::BracketOpening(..) | Token::SequenceOpening(..) | Token::BracketClosing(..) | Token::DirectiveOpening(..) | Token::ClosingTagOpening(..) | Token::DirectiveClosing(..) | Token::End(..) | Token::SequenceClosing(..) => {
                        let expression = self.parse_expression()?;
                        argument = expression.into();
                    }
                    Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
                };
            }
            Token::BracketClosing(at) | Token::SequenceClosing(at) | Token::Diamond(at) | Token::Semicolon(at) | Token::DirectiveClosing(at) | Token::End(at) => {
                argument = ParsedExpression::Empty(at.clone(), at.clone());
            }
            Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
        };
        if !matches!(self.t, Token::BracketClosing(..)) {
            return Err(ParseError::ExpectedClosingBracket(self.position()));
        };
        self.next();
        Ok(argument)
    }

}


//// Possible parsing errors


#[derive(PartialEq, Eq, Clone)]
pub enum ParseError {
    /// Tried to escape EOS.
    EscapingEndOfStream,
    ExpectedClosingBracket(Position),
    ExpectedSequenceClosing(Position),
    ExpectedClosingAngularBracket(Position),
    ExpectedClosingSquare(Position),
    ExpectedOpeningAngularBracket(Position),
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
    MismatchedClosingTag(Position, String, Position, String),
    IllegalTagDirectiveArgument(Position),
    IllegalTagAttribute(Position),
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
        ParseError::ExpectedOpeningAngularBracket(at) => {
            format!("Expected opening angular bracket at {}:{}.", at.line, at.column)
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
            format!("Mismatched closing tag \"{}\" at {}:{} does not match opening tag \"{}\" at {}:{}.", mismatch, closing_tag_at.line, closing_tag_at.column, directive, opening_tag_at.line, opening_tag_at.column)
        }
        ParseError::IllegalTagDirectiveArgument(at) => {
            format!("Illegal tag as directive argument at {}:{}.", at.line, at.column)
        }
        ParseError::IllegalTagAttribute(at) => {
            format!("Tag is not allowed as attribute at {}:{}.", at.line, at.column)
        }
    }
}
