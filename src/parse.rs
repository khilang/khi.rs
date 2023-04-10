//! Parsing of UDL documents.
//!
//! The root element of a document can be an expression, a sequence or a dictionary. Use the
//! corresponding function to parse a document: either [parse_expression_document],
//! [parse_sequence_document] or [parse_dictionary_document].


use std::slice::Iter;
use crate::ast::{ParsedCompound, ParsedDictionary, ParsedEntry, ParsedExpression, ParsedSequence, ParsedText, ParsedAttribute, ParsedCommand};
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
            Token::SequenceOpening(position) => position,
            Token::SequenceClosing(position) => position,
            Token::Word(position, ..) => position,
            Token::Quote(position, ..) => position,
            Token::CommandOpening(position) => position,
            Token::CommandClosing(position) => position,
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
            Token::SequenceOpening(position) => position,
            Token::SequenceClosing(position) => position,
            Token::Word(position, ..) => position,
            Token::Quote(position, ..) => position,
            Token::CommandOpening(position) => position,
            Token::CommandClosing(position) => position,
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
    ///           | <acmd> <expr>
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
                            Token::BracketOpening(..) | Token::BracketClosing(..) | Token::Semicolon(..) | Token::Colon(..) | Token::SequenceOpening(..) | Token::SequenceClosing(..) | Token::Quote(..) | Token::CommandOpening(..) | Token::CommandClosing(..) | Token::End(..) => {
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
                Token::CommandOpening(..) => {
                    let command = self.parse_applied_command()?;
                    let whitespace = self.is_whitespace();
                    arguments.push((command.into(), whitespace));
                }
                Token::BracketClosing(to) | Token::SequenceClosing(to) | Token::CommandClosing(to) | Token::Semicolon(to) | Token::Colon(to) | Token::End(to) => {
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
    ///         | <>
    /// ```
    pub fn parse_sequence(&mut self) -> Result<ParsedSequence, ParseError> {
        let mut elements = vec![];
        let from = self.position();
        loop {
            let element_from = self.position();
            self.skip_whitespace();
            match self.t {
                Token::BracketOpening(..) | Token::SequenceOpening(..) | Token::CommandOpening(..) | Token::Word(..) | Token::Quote(..) => {
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
                Token::BracketClosing(position) | Token::Colon(position) | Token::SequenceClosing(position) | Token::CommandClosing(position) | Token::End(position) => {
                    return Ok(ParsedSequence { elements, from, to: position.clone() });
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
        }
    }

    /// Parse an applied command.
    ///
    /// ```text
    /// <acmd> ::= <>"(" <cmd> ")"<aarg>
    ///
    /// <aarg> ::= <>
    ///          | <>":"<word><aarg>
    ///          | <>":"<quote><aarg>
    ///          | <>":""{" <expr> "}"<aarg>
    ///          | <>":""{" <dictp> "}"<aarg>
    ///          | <>":""[" <seq> "]"<aarg>
    ///          | <>":""(" <cmd> ")"<aarg>
    ///          | <>":""(" ")"":"<><acmd>
    /// ```
    pub fn parse_applied_command(&mut self) -> Result<ParsedCommand, ParseError> {
        let from = self.position();
        if !matches!(self.t, Token::CommandOpening(..)) {
            return Err(ParseError::ExpectedOpeningParenthesis(from));
        };
        self.next();
        let (name, attributes) = self.parse_command()?;
        if !matches!(self.t, Token::CommandClosing(..)) {
            return Err(ParseError::ExpectedClosingParenthesis(self.position()));
        };
        self.next();
        let mut arguments = vec![];
        loop {
            if !matches!(self.t, Token::Colon(..)) {
                let to = self.position();
                return Ok(ParsedCommand {
                    command: name,
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
                Token::CommandOpening(..) => {
                    let command_from = self.next_position();
                    self.skip_lookahead_whitespace();
                    if matches!(self.t2, Token::CommandClosing(..)) {
                        self.next(); self.next();
                        if !matches!(self.t, Token::Colon(..)) {
                            let position = self.position();
                            return Err(ParseError::ExpectedColonAfterGroupOperator(position));
                        };
                        self.next();
                        // Gives a more specific error message.
                        if !matches!(self.t, Token::CommandOpening(..)) {
                            let position = self.position();
                            return Err(ParseError::ExpectedCommandAfterGroupOperator(position));
                        };
                        let application = self.parse_applied_command()?;
                        arguments.push(application.into());
                        let to = self.position();
                        return Ok(ParsedCommand {
                            command: name,
                            attributes,
                            arguments,
                            from,
                            to,
                        });
                    } else {
                        self.next();
                        let (command, attributes) = self.parse_command()?;
                        let command_to = self.position();
                        arguments.push(ParsedCommand {
                            command,
                            attributes,
                            arguments: vec![],
                            from: command_from,
                            to: command_to,
                        }.into());
                        if !matches!(self.t, Token::CommandClosing(..)) {
                            return Err(ParseError::ExpectedCommandClosing(command_to));
                        };
                        self.next();
                    };
                }
                Token::Whitespace(position) | Token::Comment(position) | Token::Colon(position) | Token::Semicolon(position) | Token::BracketClosing(position) | Token::SequenceClosing(position) | Token::CommandClosing(position) | Token::End(position) => {
                    return Err(ParseError::ExpectedCommandArgument(position.clone()));
                }
            };
        };
    }

    /// Parse a command.
    ///
    /// ```text
    /// <cmd> ::= <key> <attr>
    ///
    /// <attr> ::= <>
    ///          | <key> <attr>
    ///          | <key> ":" <key> <attr>
    ///          | <key> ":" "{" <expr> "}" <attr>
    ///          | <key> ":" "{" <dictp> "}" <attr>
    ///          | <key> ":" "[" <seq> "]" <attr>
    ///          | <key> ":" <app> <attr>
    /// ```
    pub fn parse_command(&mut self) -> Result<(String, Vec<ParsedAttribute>), ParseError> {
        self.skip_whitespace();
        let key = match self.t {
            Token::Word(_, k) | Token::Quote(_, k) => String::from(k),
            Token::BracketOpening(at) | Token::BracketClosing(at) | Token::Semicolon(at) | Token::Colon(at) | Token::SequenceOpening(at) | Token::SequenceClosing(at) | Token::CommandOpening(at) | Token::CommandClosing(at) | Token::End(at) => {
                return Err(ParseError::ExpectedCommandKey(at.clone()));
            }
            Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
        };
        let mut attributes = vec![];
        self.next(); self.skip_whitespace();
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
                Token::BracketOpening(..) | Token::BracketClosing(..) | Token::Semicolon(..) | Token::Colon(..) | Token::SequenceOpening(..) | Token::SequenceClosing(..) | Token::CommandOpening(..) | Token::CommandClosing(..) | Token::End(..) => {
                    return Ok((key, attributes));
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
            self.next(); self.skip_whitespace();
            match self.t {
                Token::Colon(..) => { }
                Token::BracketOpening(attribute_at) | Token::BracketClosing(attribute_at) | Token::Semicolon(attribute_at) | Token::SequenceOpening(attribute_at) | Token::SequenceClosing(attribute_at) | Token::CommandOpening(attribute_at) | Token::CommandClosing(attribute_at) | Token::End(attribute_at) | Token::Word(attribute_at, ..) | Token::Quote(attribute_at, ..) => {
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
                Token::CommandOpening(..) => {
                    let application = self.parse_applied_command()?;
                    application.into()
                }
                Token::BracketClosing(at) | Token::Semicolon(at) | Token::Colon(at) | Token::SequenceClosing(at) | Token::CommandClosing(at) | Token::End(at) => {
                    return Err(ParseError::ExpectedAttributeArgument(at.clone()));
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
            attributes.push(ParsedAttribute { key, value });
            self.skip_whitespace();
        };
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
                Token::BracketOpening(to) | Token::CommandOpening(to) | Token::CommandClosing(to) | Token::BracketClosing(to) | Token::End(to) | Token::Semicolon(to) | Token::SequenceOpening(to) | Token::SequenceClosing(to) => {
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
                Token::BracketOpening(at) | Token::BracketClosing(at) | Token::SequenceOpening(at) | Token::SequenceClosing(at) | Token::Word(at, ..) | Token::Quote(at, ..) | Token::CommandOpening(at) | Token::CommandClosing(at) | Token::End(at) => {
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
                Token::Word(to, ..) | Token::Quote(to, ..) | Token::BracketOpening(to) | Token::SequenceOpening(to) | Token::CommandOpening(to) | Token::End(to) | Token::BracketClosing(to) | Token::SequenceClosing(to) | Token::CommandClosing(to) | Token::Colon(to) => {
                    return Ok(ParsedDictionary { entries, from, to: to.clone() });
                }
                Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
            };
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
            Token::SequenceOpening(..) | Token::BracketOpening(..) | Token::CommandOpening(..) => {
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
                    Token::Word(..) | Token::Quote(..) | Token::BracketOpening(..) | Token::SequenceOpening(..) | Token::BracketClosing(..) | Token::CommandOpening(..) | Token::CommandClosing(..) | Token::End(..) | Token::SequenceClosing(..) => {
                        let expression = self.parse_expression()?;
                        argument = expression.into();
                    }
                    Token::Whitespace(..) | Token::Comment(..) => unreachable!(),
                };
            }
            Token::BracketClosing(at) | Token::SequenceClosing(at) | Token::Semicolon(at) | Token::CommandClosing(at) | Token::End(at) => {
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
    ExpectedClosingParenthesis(Position),
    ExpectedClosingSquare(Position),
    ExpectedOpeningParenthesis(Position),
    ExpectedColonAfterGroupOperator(Position),
    ExpectedCommandAfterGroupOperator(Position),
    ExpectedCommandClosing(Position),
    ExpectedCommandArgument(Position),
    ExpectedCommandKey(Position),
    ExpectedAttributeArgument(Position),
    ExpectedEntrySeparator(Position),
    ExpectedEnd(Position),
    CommentedBracket(Position),
    UnclosedQuote(Position),
}
