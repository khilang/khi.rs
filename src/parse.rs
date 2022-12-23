//! Parsing of Dx documents.
//!
//! The root node of a Dx document may be an open expression, an open sequence or an open
//! dictionary. Parse such a document with the corresponding function: [parse_expression],
//! [parse_sequence] or [parse_dictionary].
//!
//! This is a recursive implementation. To deeply nested documents may result in stack overflow.


use std::fmt::{Display, Formatter};
use std::slice::Iter;
use std::str::Chars;


//// Parsing functions
////
//// A user uses these functions to parse a Dx document.


/// Parse an expression.
///
/// Assumes that the document root is an open expression.
pub fn parse_expression(document: &str) -> Result<ParsedExpression, ParseError> {
    let mut iter = CharIter::new(document.chars());
    iter.parse_open_expression()
}


/// Parse a sequence.
///
/// Assumes that the document root is an open sequence.
pub fn parse_sequence(document: &str) -> Result<ParsedSequence, ParseError> {
    let mut iter = CharIter::new(document.chars());
    iter.parse_open_sequence()
}


/// Parse a dictionary.
///
/// Assumes that the document root is an open dictionary.
pub fn parse_dictionary(document: &str) -> Result<ParsedDictionary, ParseError> {
    let mut iter = CharIter::new(document.chars());
    iter.parse_open_dictionary()
}


//// Parsing results
////
//// Parsing a document yields a nested structure consisting of these structures.


/// A position.
///
/// Contains a line number and a column number, corresponding to a character in a document.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Position { pub index: usize, pub line: usize, pub column: usize }


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedExpression {
    pub(crate) arguments: Vec<ParsedArgument>,
}


impl ParsedExpression {

    pub fn empty() -> Self {
        ParsedExpression { arguments: vec![] }
    }

    pub fn arguments(&self) -> &[ParsedArgument] {
        &self.arguments
    }

    pub fn length(&self) -> usize {
        self.arguments.len()
    }

    pub fn nth_argument(&self, n: usize) -> Option<&ParsedArgument> {
        self.arguments.get(n)
    }

    pub fn first_argument(&self) -> Option<&ParsedArgument> {
        self.nth_argument(0)
    }

    pub fn last_argument(&self) -> Option<&ParsedArgument> {
        self.nth_argument(self.length() - 1)
    }

    pub fn nth_last_argument(&self, n: usize) -> Option<&ParsedArgument> {
        self.nth_argument(self.length() - n - 1)
    }

    pub fn iter(&self) -> Iter<ParsedArgument> {
        self.arguments.iter()
    }

}


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedFunctionExpression {
    arguments: Vec<ParsedFunctionArgument>,
}


#[derive(PartialEq, Eq, Clone)]
pub enum ParsedArgument {
    Symbol(ParsedSymbol),
    Quote(ParsedQuote),
    Sequence(ParsedSequence),
    Dictionary(ParsedDictionary),
    Grouping(ParsedGrouping),
    Function(ParsedFunction),
}


impl ParsedArgument {

    pub fn from(&self) -> Position {
        match self {
            ParsedArgument::Symbol(ParsedSymbol { from, .. }) => from,
            ParsedArgument::Quote(ParsedQuote { from, .. }) => from,
            ParsedArgument::Sequence(ParsedSequence { from, .. }) => from,
            ParsedArgument::Dictionary(ParsedDictionary { from, .. }) => from,
            ParsedArgument::Grouping(ParsedGrouping { from, .. }) => from,
            ParsedArgument::Function(ParsedFunction { from, .. }) => from,
        }.clone()
    }

    pub fn to(&self) -> Position {
        match self {
            ParsedArgument::Symbol(ParsedSymbol { to, .. }) => to,
            ParsedArgument::Quote(ParsedQuote { to, .. }) => to,
            ParsedArgument::Sequence(ParsedSequence { to, .. }) => to,
            ParsedArgument::Dictionary(ParsedDictionary { to, .. }) => to,
            ParsedArgument::Grouping(ParsedGrouping { to, .. }) => to,
            ParsedArgument::Function(ParsedFunction { to, .. }) => to,
        }.clone()
    }

}


impl From<ParsedSymbol> for ParsedArgument {

    fn from(symbol: ParsedSymbol) -> Self {
        ParsedArgument::Symbol(symbol)
    }

}


impl From<ParsedQuote> for ParsedArgument {

    fn from(quote: ParsedQuote) -> Self {
        ParsedArgument::Quote(quote)
    }

}


impl From<ParsedSequence> for ParsedArgument {

    fn from(sequence: ParsedSequence) -> Self {
        ParsedArgument::Sequence(sequence)
    }

}


impl From<ParsedDictionary> for ParsedArgument {

    fn from(dictionary: ParsedDictionary) -> Self {
        ParsedArgument::Dictionary(dictionary)
    }

}


impl From<ParsedGrouping> for ParsedArgument {

    fn from(grouping: ParsedGrouping) -> Self {
        ParsedArgument::Grouping(grouping)
    }

}


impl From<ParsedFunction> for ParsedArgument {

    fn from(function: ParsedFunction) -> Self {
        ParsedArgument::Function(function)
    }

}


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedSymbol { pub symbol: String, pub from: Position, pub to: Position }


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedQuote { pub quote: String, pub from: Position, pub to: Position }


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedKey { pub key: String, pub from: Position, pub to: Position }


impl From<ParsedSymbol> for ParsedKey {

    fn from(glyphs: ParsedSymbol) -> Self {
        ParsedKey { key: glyphs.symbol, from: glyphs.from, to: glyphs.to }
    }

}


impl From<ParsedQuote> for ParsedKey {

    fn from(quote: ParsedQuote) -> Self {
        ParsedKey { key: quote.quote, from: quote.from, to: quote.to }
    }

}


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedSequence { pub elements: Vec<ParsedExpression>, pub from: Position, pub to: Position }


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedEntry { pub key: ParsedKey, pub value: ParsedExpression }


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedDictionary { pub entries: Vec<ParsedEntry>, pub from: Position, pub to: Position }


impl ParsedDictionary {

    pub fn empty(from: Position, to: Position) -> Self {
        ParsedDictionary { entries: vec![], from, to }
    }

}


#[derive(PartialEq, Eq, Clone)]
pub enum ParsedFunctionArgument {
    Positional { argument: ParsedArgument },
    Option { key: ParsedKey, value: ParsedArgument },
    Flag { flag: ParsedKey },
}


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedGrouping { pub expression: ParsedExpression, pub from: Position, pub to: Position }


impl ParsedGrouping {

    pub fn new(expression: ParsedExpression, from: Position, to: Position ) -> Self {
        ParsedGrouping { expression, from, to }
    }

    fn empty_from_underscore(position: Position) -> Self {
        ParsedGrouping::new(ParsedExpression::empty(), position, position)
    }

}


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedFunction { pub arguments: Vec<ParsedFunctionArgument>, pub from: Position, pub to: Position }


//// Parsing result formatting


/// Check if a string has reserved characters. Such a string must be escaped.
pub fn has_reserved(string: &str) -> bool {
    string.contains(|c: char| {
        c.is_whitespace() || c == ':' || c == ';' || c == '(' || c == '[' || c == '{' || c == '⟨' ||
        c == ')' || c == ']' || c == '}' || c == '⟩' || c == '"' || c == '\\'
    })
}


impl Display for ParsedExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let len = self.arguments.len();
        if len > 0 {
            write!(f, "{}", self.arguments.get(0).unwrap())?;
        };
        let mut i = 1;
        while i < self.arguments.len() {
            write!(f, " {}", self.arguments.get(i).unwrap())?;
            i += 1;
        };
        Ok(())
    }
}


impl Display for ParsedArgument {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsedArgument::Symbol(s) => s.fmt(f),
            ParsedArgument::Quote(q) => q.fmt(f),
            ParsedArgument::Function(ff) => ff.fmt(f),
            ParsedArgument::Sequence(s) => s.fmt(f),
            ParsedArgument::Dictionary(d) => d.fmt(f),
            ParsedArgument::Grouping(g) => g.fmt(f),
        }
    }

}


impl Display for ParsedSymbol {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let reserved = has_reserved(&self.symbol);
        if reserved {
            write!(f, "⟨{}⟩", self.symbol)
        } else {
            write!(f, "{}", self.symbol)
        }
    }

}


impl Display for ParsedQuote {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.quote)
    }

}


impl Display for ParsedKey {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "⟨{}⟩", self.key)
    }

}


impl Display for ParsedSequence {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for a in &self.elements {
            a.fmt(f)?;
            write!(f, "; ")?;
        }
        write!(f, "]")?;
        Ok(())
    }

}


impl Display for ParsedEntry {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.key.fmt(f)?;
        write!(f, ": ")?;
        self.value.fmt(f)?;
        write!(f, "; ")?;
        Ok(())
    }

}


impl Display for ParsedDictionary {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for e in &self.entries {
            e.fmt(f)?;
        }
        write!(f, "}}")?;
        Ok(())
    }

}


impl Display for ParsedGrouping {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        self.expression.fmt(f)?;
        write!(f, "}}")?;
        Ok(())
    }

}


impl Display for ParsedFunctionArgument {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsedFunctionArgument::Positional { argument } => {
                argument.fmt(f)?;
            }
            ParsedFunctionArgument::Option { key, value } => {
                key.fmt(f)?;
                write!(f, ":")?;
                value.fmt(f)?;
            }
            ParsedFunctionArgument::Flag { flag } => {
                flag.fmt(f)?;
                write!(f, ";")?;
            }
        };
        Ok(())
    }

}



impl Display for ParsedFunction {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        let len = self.arguments.len();
        if len > 0 {
            self.arguments.first().unwrap().fmt(f)?;
        };
        let mut i = 1;
        while i < len {
            let a = self.arguments.get(i).unwrap();
            write!(f, " ")?;
            a.fmt(f)?;
            i += 1;
        }
        write!(f, ")")?;
        Ok(())
    }

}


//// Possible parsing errors


#[derive(PartialEq, Eq, Clone)]
pub struct ParseError { pub position: Position, pub error: ErrorType }


#[derive(PartialEq, Eq, Clone)]
pub enum ErrorType {
    /// The closing character does not match the opening symbol.
    ClosingMismatch,
    /// Tried to escape EOS.
    EscapingEndOfStream,
    /// A semicolon delimiter is not allowed here.
    IllegalSemicolon,
    /// Invalid dictionary key.
    InvalidKey,
    /// A colon delimiter is not allowed here.
    IllegalColon,
    /// Expected an option value.
    OptionNotFinished,
    /// Expected a colon after key argument.
    ExpectedColon,
}


//// Intermediary parse returns
////
//// Return values used to communicate between the recursive functions.


enum ExpressionParse {
    ExpressionThenDelimiter { expression: ParsedExpression },
    ExpressionThenClosing { expression: ParsedExpression },
}


enum GlyphsParse {
    Symbol { symbol: ParsedSymbol, closing: GlyphsClosing },
    EmptyGrouping { position: Position, closing: GlyphsClosing },
    Comment,
}


/// Cause of glyphs termination.
enum GlyphsClosing {
    Whitespace,
    Colon,
    Semicolon,
    Closing,
    EnclosureOpening,
    QuoteOpening,
    SequenceOpening,
    CurlyBracketOpening,
    FunctionOpening,
}


enum CurlyBracketsParse {
    Dictionary(ParsedDictionary),
    Grouping(ParsedGrouping),
}


/// Cause of seek return.
#[derive(Eq, PartialEq)]
enum SeekResult {
    Glyphs(char),
    Colon,
    Semicolon,
    Closing,
    EnclosureOpening,
    QuoteOpening,
    SequenceOpening,
    CurlyBracketOpening,
    FunctionOpening,
}


/// An argument that can be a key.
#[derive(PartialEq, Eq, Clone)]
enum KeyArgument {
    Symbol(ParsedSymbol),
    Quote(ParsedQuote)
}


impl From<ParsedSymbol> for KeyArgument {

    fn from(symbol: ParsedSymbol) -> Self {
        KeyArgument::Symbol(symbol)
    }

}


impl From<ParsedQuote> for KeyArgument {

    fn from(quote: ParsedQuote) -> Self {
        KeyArgument::Quote(quote)
    }

}


impl From<KeyArgument> for ParsedKey {

    fn from(a: KeyArgument) -> Self {
        match a {
            KeyArgument::Symbol(g) => g.into(),
            KeyArgument::Quote(q) => q.into(),
        }
    }

}


impl From<KeyArgument> for ParsedArgument {

    fn from(a: KeyArgument) -> Self {
        match a {
            KeyArgument::Symbol(g) => ParsedArgument::Symbol(g),
            KeyArgument::Quote(q) => ParsedArgument::Quote(q),
        }
    }

}


/// Possible states of the state machine used to parse an expression.
enum ExpressionParseState {
    Seek,
    Glyphs(char),
    Colon,
    Semicolon,
    Closing,
    Enclosure,
    Quote,
    Sequence,
    CurlyBracket,
    Function,
}


//// Character iterator


/// Character iterator keeping track of current index, line and column.
struct CharIter<'a> {
    chars: Chars<'a>,
    current: Option<char>,
    index: usize,
    line: usize,
    position: usize,
}


impl<'a> CharIter<'a> {

    fn new(mut chars: Chars<'a>) -> Self {
        let current = None;
        CharIter {
            chars,
            current,
            index: usize::MAX, // First character will have index 0.
            line: 1,
            position: 0,
        }
    }

    /// Retrieve the current element.
    ///
    /// The current element will be [None] if [Self::next] has not been called yet, or if EOS is
    /// reached.
    fn current(&self) -> Option<char> {
        self.current
    }

    /// Advances the iterator and returns the next value.
    fn next(&mut self) -> Option<char> {
        if let Some(c) = self.current {
            if c == '\n' {
                self.current = self.chars.next();
                self.position = 1;
                self.line += 1;
                self.current
            } else {
                self.current = self.chars.next();
                self.position += 1;
                self.current
            }
        } else {
            self.current = self.chars.next();
            self.position += 1;
            self.current //todo
        }
    }

    /// Skip until reaching a newline.
    fn skip_line(&mut self) {
        loop {
            if let Some(c) = self.next() {
                if c == '\n' {
                    return;
                }
            } else {
                return; //todo: cause crash?
            }
        }
    }

    fn position(&self) -> Position {
        Position { index: self.index, line: self.line, column: self.position }
    }

    fn index(&self) -> usize {
        self.index
    }

}


impl<'a> CharIter<'a> {

    //// Expression

    /// Parse an expression until EOS.
    ///
    /// Assumes that the iterator is at the opening of the expression.
    fn parse_open_expression(&mut self) -> Result<ParsedExpression, ParseError> {
        match self.parse_expression_tail(None, vec![], ExpressionParseState::Seek)? {
            ExpressionParse::ExpressionThenDelimiter { .. } => self.error_at_position(ErrorType::IllegalSemicolon),
            ExpressionParse::ExpressionThenClosing { expression } => Ok(expression),
        }
    }

    /// Parse an expression tail in the given state.
    ///
    /// Assumes that the iterator is at the end of an argument.
    fn parse_expression_tail(&mut self, expected_closing: Option<char>, head: Vec<ParsedArgument>, state: ExpressionParseState) -> Result<ExpressionParse, ParseError> {
        let mut arguments = head;
        let mut state = state;
        loop {
            let s = match state {
                ExpressionParseState::Seek => {
                    match self.seek(expected_closing)? {
                        SeekResult::Closing => ExpressionParseState::Closing,
                        SeekResult::FunctionOpening => ExpressionParseState::Function,
                        SeekResult::SequenceOpening => ExpressionParseState::Sequence,
                        SeekResult::CurlyBracketOpening => ExpressionParseState::CurlyBracket,
                        SeekResult::QuoteOpening => ExpressionParseState::Quote,
                        SeekResult::EnclosureOpening => ExpressionParseState::Enclosure,
                        SeekResult::Colon => ExpressionParseState::Colon,
                        SeekResult::Semicolon => ExpressionParseState::Semicolon,
                        SeekResult::Glyphs(initial) => ExpressionParseState::Glyphs(initial),
                    }
                }
                ExpressionParseState::Function => {
                    let function = self.parse_closed_function()?;
                    arguments.push(function.into());
                    ExpressionParseState::Seek
                }
                ExpressionParseState::Sequence => {
                    let sequence = self.parse_closed_sequence()?;
                    arguments.push(sequence.into());
                    ExpressionParseState::Seek
                }
                ExpressionParseState::CurlyBracket => {
                    match self.parse_closed_curly_bracket()? {
                        CurlyBracketsParse::Dictionary(dictionary) => arguments.push(dictionary.into()),
                        CurlyBracketsParse::Grouping(grouping) => arguments.push(grouping.into()),
                    };
                    ExpressionParseState::Seek
                }
                ExpressionParseState::Quote => {
                    let quote = self.parse_double_quote()?;
                    arguments.push(quote.into());
                    ExpressionParseState::Seek
                }
                ExpressionParseState::Colon => {
                    return self.error_at_position(ErrorType::IllegalColon);
                }
                ExpressionParseState::Semicolon => {
                    return Ok(ExpressionParse::ExpressionThenDelimiter { expression: ParsedExpression { arguments } });
                }
                ExpressionParseState::Glyphs(c) => {
                    let closing = match self.parse_glyphs(c, expected_closing)? {
                        GlyphsParse::Symbol { symbol, closing } => {
                            arguments.push(symbol.into());
                            closing
                        }
                        GlyphsParse::EmptyGrouping { position, closing } => {
                            let grouping = ParsedGrouping { expression: ParsedExpression::empty(), from: position, to: position };
                            arguments.push(grouping.into());
                            closing
                        }
                        GlyphsParse::Comment => GlyphsClosing::Whitespace,
                    };
                    match closing {
                        GlyphsClosing::Whitespace => ExpressionParseState::Seek,
                        GlyphsClosing::Closing => ExpressionParseState::Closing,
                        GlyphsClosing::FunctionOpening => ExpressionParseState::Function,
                        GlyphsClosing::SequenceOpening => ExpressionParseState::Sequence,
                        GlyphsClosing::CurlyBracketOpening => ExpressionParseState::CurlyBracket,
                        GlyphsClosing::QuoteOpening => ExpressionParseState::Quote,
                        GlyphsClosing::EnclosureOpening => ExpressionParseState::Enclosure,
                        GlyphsClosing::Colon => ExpressionParseState::Colon,
                        GlyphsClosing::Semicolon => ExpressionParseState::Semicolon,
                    }
                }
                ExpressionParseState::Enclosure => {
                    let symbol = self.parse_enclosed_symbol()?;
                    arguments.push(symbol.into());
                    ExpressionParseState::Seek
                }
                ExpressionParseState::Closing => {
                    return Ok(ExpressionParse::ExpressionThenClosing { expression: ParsedExpression { arguments } });
                }
            };
            state = s;
        };
    }

    //// Glyphs

    /// Parse a sequence of glyphs.
    ///
    /// Assumes that the iterator is at the first glyph.
    ///
    /// Reads until reaching a delimiter, opening, closing, whitespace, or EOS.
    fn parse_glyphs(&mut self, initial: char, expected_closing: Option<char>) -> Result<GlyphsParse, ParseError> {
        let mut glyphs = String::new();
        let from = self.position();
        fn result_at_opening_after_glyphs(iter: &mut CharIter, glyphs: String, from: Position, closing: GlyphsClosing) -> Result<GlyphsParse, ParseError> {
            let to = iter.position();
            return Ok(GlyphsParse::Symbol { symbol: ParsedSymbol { symbol: glyphs, from, to }, closing });
        }
        fn f(iter: &CharIter, c: char) {
            if c.is_whitespace() || c == '#' || c == ':' || c == ';' || c == ')' || c == ']' || c == '}' || c == '⟩' || c == '(' || c == '[' || c == '{' || c == '⟨' || c == '"' {

            }
        }
        if initial == '#' {
            if let Some(d) = self.next() {
                if d.is_whitespace() || d == '#' || d == ':' || d == ';' || d == ')' || d == ']' || d == '}' || d == '⟩' || d == '(' || d == '[' || d == '{' || d == '⟨' || d == '"' {
                    self.skip_line();
                    return Ok(GlyphsParse::Comment);
                } else if d == '\\' {
                    glyphs.push('#');
                    if let Some(e) = self.next() {
                        glyphs.push(e);
                    } else {
                        return self.error_at_position(ErrorType::EscapingEndOfStream);
                    };
                } else {
                    glyphs.push('#');
                    glyphs.push(d);
                }
            } else {
                return self.result_at_end_of_stream(expected_closing, GlyphsParse::Comment);
            };
        } else if initial == '_' {
            let position = self.position();
            if let Some(d) = self.next() {
                if d == ':' {
                    return Ok(GlyphsParse::EmptyGrouping { closing: GlyphsClosing::Colon, position });
                } else if d == ';' {
                    return Ok(GlyphsParse::EmptyGrouping { closing: GlyphsClosing::Semicolon, position });
                } else if d == ')' || d == ']' || d == '}' || d == '⟩' {
                    return if Some(d) == expected_closing {
                        Ok(GlyphsParse::EmptyGrouping { closing: GlyphsClosing::Closing, position })
                    } else {
                        self.error_at_position(ErrorType::ClosingMismatch)
                    };
                } else if d == '(' {
                    return Ok(GlyphsParse::EmptyGrouping { closing: GlyphsClosing::FunctionOpening, position });
                } else if d == '[' {
                    return Ok(GlyphsParse::EmptyGrouping { closing: GlyphsClosing::SequenceOpening, position });
                } else if d == '{' {
                    return Ok(GlyphsParse::EmptyGrouping { closing: GlyphsClosing::CurlyBracketOpening, position });
                } else if d == '⟨' {
                    return Ok(GlyphsParse::EmptyGrouping { closing: GlyphsClosing::EnclosureOpening, position });
                } else if d == '"' {
                    return Ok(GlyphsParse::EmptyGrouping { closing: GlyphsClosing::QuoteOpening, position });
                } else if d == '\\' {
                    if let Some(e) = self.next() {
                        glyphs.push('_');
                        glyphs.push(e);
                    } else {
                        return self.error_at_position(ErrorType::EscapingEndOfStream);
                    };
                } else if d.is_whitespace() {
                    return Ok(GlyphsParse::EmptyGrouping { closing: GlyphsClosing::Whitespace, position });
                } else {
                    glyphs.push('_');
                    glyphs.push(d);
                };
            } else {
                return self.result_at_end_of_stream(expected_closing, GlyphsParse::EmptyGrouping { position, closing: GlyphsClosing::Whitespace });//todo
            }
        } else if initial == '\\' {
            if let Some(d) = self.next() {
                glyphs.push(d);
            } else {
                return self.error_at_position(ErrorType::EscapingEndOfStream);
            }
        } else {
            glyphs.push(initial);
        };
        loop {
            if let Some(c) = self.next() {
                if c.is_whitespace() {
                    let to = self.position();
                    return Ok(GlyphsParse::Symbol { symbol: ParsedSymbol { symbol: glyphs, from, to }, closing: GlyphsClosing::Whitespace});
                } else if c == ':' {
                    let to = self.position();
                    return Ok(GlyphsParse::Symbol { symbol: ParsedSymbol { symbol: glyphs, from, to }, closing: GlyphsClosing::Colon });
                } else if c == ';' {
                    let to = self.position();
                    return Ok(GlyphsParse::Symbol { symbol: ParsedSymbol { symbol: glyphs, from, to }, closing: GlyphsClosing::Semicolon });
                } else if c == '(' {
                    return result_at_opening_after_glyphs(self, glyphs, from, GlyphsClosing::FunctionOpening);
                } else if c == '[' {
                    return result_at_opening_after_glyphs(self, glyphs, from, GlyphsClosing::SequenceOpening);
                } else if c == '{' {
                    return result_at_opening_after_glyphs(self, glyphs, from, GlyphsClosing::CurlyBracketOpening);
                } else if c == '⟨' {
                    return result_at_opening_after_glyphs(self, glyphs, from, GlyphsClosing::EnclosureOpening);
                } else if c == '"' {
                    return result_at_opening_after_glyphs(self, glyphs, from, GlyphsClosing::QuoteOpening);
                } else if c == ')' || c == ']' || c == '}' || c == '⟩' {
                    let to = self.position();
                    return self.result_at_closing_bracket(c, expected_closing, GlyphsParse::Symbol { symbol: ParsedSymbol { symbol: glyphs, from, to }, closing: GlyphsClosing::Closing });
                } else if c == '\\' {
                    if let Some(d) = self.next() {
                        glyphs.push(d);
                    } else {
                        let escape_position = self.position();
                        let escape_position = Position { index: escape_position.index - 1, line: escape_position.line, column: escape_position.column - 1 };
                        return Err(ParseError { position: escape_position, error: ErrorType::EscapingEndOfStream });
                    };
                } else {
                    glyphs.push(c);
                };
            } else {
                let to = self.position();
                return self.result_at_end_of_stream(expected_closing, GlyphsParse::Symbol { symbol: ParsedSymbol { symbol: glyphs, from, to }, closing: GlyphsClosing::Closing });
            };
        };
    }

    //// Symbol

    /// Parse a symbol enclosed in chevrons `⟨`, `⟩`.
    ///
    /// Assumes that the iterator is at the opening chevron.
    fn parse_enclosed_symbol(&mut self) -> Result<ParsedSymbol, ParseError> {
        let from = self.position();
        let ParsedQuote { quote, from, to } = self.parse_enclosed_characters('⟩', from)?;
        Ok(ParsedSymbol { symbol: quote, from, to })
    }

    //// Quote

    /// Parse a quote enclosed in double quotes.
    ///
    /// Assumes that the iterator is at the opening quote.
    fn parse_double_quote(&mut self) -> Result<ParsedQuote, ParseError> {
        let from = self.position();
        self.parse_enclosed_characters('"', from)
    }

    //// Sequence

    /// Parse an open sequence.
    fn parse_open_sequence(&mut self) -> Result<ParsedSequence, ParseError> {
        self.parse_sequence(None)
    }

    /// Parse a closed sequence.
    fn parse_closed_sequence(&mut self) -> Result<ParsedSequence, ParseError> {
        self.parse_sequence(Some(']'))
    }

    /// Parse a sequence.
    ///
    /// Assumes that the iterator is at the opening square bracket.
    fn parse_sequence(&mut self, expected_closing: Option<char>) -> Result<ParsedSequence, ParseError> {
        let mut elements = vec![];
        let from = self.position();
        loop {
            match self.parse_expression_tail(expected_closing, vec![], ExpressionParseState::Seek)? {
                ExpressionParse::ExpressionThenDelimiter { expression } => elements.push(expression),
                ExpressionParse::ExpressionThenClosing { expression } => {
                    elements.push(expression);
                    break;
                }
            };
        };
        let to = self.position();
        Ok(ParsedSequence { elements, from, to })
    }

    //// Curly bracket

    /// Parse a closed curly bracket.
    ///
    /// A curly bracket argument is either a dictionary or a grouping, but this is not known until
    /// the first argument has been read.
    fn parse_closed_curly_bracket(&mut self) -> Result<CurlyBracketsParse, ParseError> {
        fn result_at_nonkey_argument(iter: &mut CharIter, potential_state: PotentialState, from: Position, argument: ParsedArgument) -> Result<CurlyBracketsParse, ParseError> {
            let head = match potential_state {
                PotentialState::None => vec![argument],
                PotentialState::Potential(potential) => vec![potential.into(), argument],
            };
            Ok(CurlyBracketsParse::Grouping(iter.parse_closed_grouping_tail_in_state(head, from, ExpressionParseState::Seek)?))
        }
        fn result_at_two_arguments(iter: &mut CharIter, potential: KeyArgument, from: Position, argument: ParsedArgument, parse_state: ExpressionParseState) -> Result<CurlyBracketsParse, ParseError> {
            let head = vec![potential.into(), argument];
            Ok(CurlyBracketsParse::Grouping(iter.parse_closed_grouping_tail_in_state(head, from, parse_state)?))
        }
        enum ParseState { Seek, Closing, Function, Sequence, CurlyBracket, Quote, Enclosure, Colon, Semicolon, Glyphs(char) }
        enum PotentialState { None, Potential(KeyArgument) }
        let from = self.position();
        let mut parse_state = ParseState::Seek;
        let mut potential_state = PotentialState::None;
        loop {
            let (a, b) = match parse_state {
                ParseState::Seek => {
                    let parse_state = match self.seek(Some('}'))? {
                        SeekResult::Closing => ParseState::Closing,
                        SeekResult::FunctionOpening => ParseState::Function,
                        SeekResult::SequenceOpening => ParseState::Sequence,
                        SeekResult::CurlyBracketOpening => ParseState::CurlyBracket,
                        SeekResult::QuoteOpening => ParseState::Quote,
                        SeekResult::EnclosureOpening => ParseState::Enclosure,
                        SeekResult::Colon => ParseState::Colon,
                        SeekResult::Semicolon => ParseState::Semicolon,
                        SeekResult::Glyphs(initial) => ParseState::Glyphs(initial),
                    };
                    (parse_state, potential_state)
                }
                ParseState::Closing => {
                    let to = self.position();
                    return match potential_state {
                        PotentialState::None => {
                            let dictionary = ParsedDictionary { entries: vec![], from, to };
                            Ok(CurlyBracketsParse::Dictionary(dictionary))
                        }
                        PotentialState::Potential(potential) => {
                            let grouping = ParsedGrouping { expression: ParsedExpression { arguments: vec![potential.into()] }, from, to };
                            Ok(CurlyBracketsParse::Grouping(grouping))
                        }
                    };
                }
                ParseState::Function => {
                    let function = self.parse_closed_function()?;
                    return result_at_nonkey_argument(self, potential_state, from, function.into());
                }
                ParseState::Sequence => {
                    let sequence = self.parse_closed_sequence()?;
                    return result_at_nonkey_argument(self, potential_state, from, sequence.into());
                }
                ParseState::CurlyBracket => {
                    let argument = match self.parse_closed_curly_bracket()? {
                        CurlyBracketsParse::Dictionary(dictionary) => dictionary.into(),
                        CurlyBracketsParse::Grouping(grouping) => grouping.into(),
                    };
                    return result_at_nonkey_argument(self, potential_state, from, argument);
                }
                ParseState::Quote => {
                    let quote = self.parse_double_quote()?;
                    match potential_state {
                        PotentialState::None => (ParseState::Seek, PotentialState::Potential(quote.into())),
                        PotentialState::Potential(potential) => return result_at_two_arguments(self, potential, from, quote.into(), ExpressionParseState::Seek),
                    }
                }
                ParseState::Enclosure => {
                    let symbol = self.parse_enclosed_symbol()?;
                    match potential_state {
                        PotentialState::None => (ParseState::Seek, PotentialState::Potential(symbol.into())),
                        PotentialState::Potential(potential) => return result_at_two_arguments(self, potential, from, symbol.into(), ExpressionParseState::Seek),
                    }
                }
                ParseState::Colon => {
                    return match potential_state {
                        PotentialState::None => self.error_at_position(ErrorType::IllegalColon),
                        PotentialState::Potential(potential) => {
                            match self.parse_expression_tail(Some('}'), vec![], ExpressionParseState::Seek)? {
                                ExpressionParse::ExpressionThenDelimiter { expression } => {
                                    let entry = ParsedEntry { key: potential.into(), value: expression };
                                    let dictionary = self.parse_closed_dictionary_tail(from, vec![entry])?;
                                    Ok(CurlyBracketsParse::Dictionary(dictionary.into()))
                                }
                                ExpressionParse::ExpressionThenClosing { expression } => {
                                    let to = self.position();
                                    let entry = ParsedEntry { key: potential.into(), value: expression };
                                    let dictionary = ParsedDictionary { entries: vec![entry], from, to };
                                    Ok(CurlyBracketsParse::Dictionary(dictionary))
                                }
                            }
                        }
                    }
                }
                ParseState::Semicolon => return self.error_at_position(ErrorType::IllegalSemicolon),
                ParseState::Glyphs(c) => {
                    match self.parse_glyphs(c, Some('}'))? {
                        GlyphsParse::Symbol { symbol, closing } => {
                            let potential = match potential_state {
                                PotentialState::None => PotentialState::Potential(symbol.into()),
                                PotentialState::Potential(potential) => {
                                    let parse_state = match closing {
                                        GlyphsClosing::Whitespace => ExpressionParseState::Seek,
                                        GlyphsClosing::Closing => ExpressionParseState::Closing,
                                        GlyphsClosing::FunctionOpening => ExpressionParseState::Function,
                                        GlyphsClosing::SequenceOpening => ExpressionParseState::Sequence,
                                        GlyphsClosing::CurlyBracketOpening => ExpressionParseState::CurlyBracket,
                                        GlyphsClosing::QuoteOpening => ExpressionParseState::Quote,
                                        GlyphsClosing::EnclosureOpening => ExpressionParseState::Enclosure,
                                        GlyphsClosing::Colon => ExpressionParseState::Colon,
                                        GlyphsClosing::Semicolon => ExpressionParseState::Semicolon,
                                    };
                                    return result_at_two_arguments(self, potential, from, symbol.into(), parse_state)
                                },
                            };
                            let state = match closing {
                                GlyphsClosing::Whitespace => ParseState::Seek,
                                GlyphsClosing::Closing => ParseState::Closing,
                                GlyphsClosing::FunctionOpening => ParseState::Function,
                                GlyphsClosing::SequenceOpening => ParseState::Sequence,
                                GlyphsClosing::CurlyBracketOpening => ParseState::CurlyBracket,
                                GlyphsClosing::QuoteOpening => ParseState::Quote,
                                GlyphsClosing::EnclosureOpening => ParseState::Enclosure,
                                GlyphsClosing::Colon => ParseState::Colon,
                                GlyphsClosing::Semicolon => ParseState::Semicolon,
                            };
                            (state, potential)
                        }
                        GlyphsParse::EmptyGrouping { position, .. } => return result_at_nonkey_argument(self, potential_state, from, ParsedGrouping::empty_from_underscore(position).into()),
                        GlyphsParse::Comment => (ParseState::Seek, potential_state),
                    }
                }
            };
            parse_state = a;
            potential_state = b;
        }
    }

    //// Dictionary

    /// Parse a dictionary argument until EOS.
    fn parse_open_dictionary(&mut self) -> Result<ParsedDictionary, ParseError> {
        let from = self.position();
        self.parse_closed_dictionary_tail(from, vec![])
    }

    /// Parse a dictionary argument tail.
    fn parse_closed_dictionary_tail(&mut self, from: Position, head: Vec<ParsedEntry>) -> Result<ParsedDictionary, ParseError> {
        let mut entries = head;
        loop {
            let (key, found) = match self.seek(Some('}'))? {
                SeekResult::Glyphs(initial) => {
                    match self.parse_glyphs(initial, Some('}'))? {
                        GlyphsParse::Symbol { symbol, closing } => {
                            match closing {
                                GlyphsClosing::Whitespace => (symbol.into(), false),
                                GlyphsClosing::Colon => (symbol.into(), true),
                                _ => return self.error_at_position(ErrorType::ExpectedColon),
                            }
                        }
                        GlyphsParse::EmptyGrouping { .. } => return self.error_at_position(ErrorType::InvalidKey),
                        GlyphsParse::Comment => continue,
                    }
                }
                SeekResult::EnclosureOpening => (self.parse_enclosed_symbol()?.into(), false),
                SeekResult::QuoteOpening => (self.parse_double_quote()?.into(), false),
                SeekResult::Closing => return Ok(ParsedDictionary { entries, from, to: self.position() }),
                SeekResult::FunctionOpening | SeekResult::SequenceOpening | SeekResult::CurlyBracketOpening => return self.error_at_position(ErrorType::InvalidKey),
                SeekResult::Colon => return self.error_at_position(ErrorType::IllegalColon),
                SeekResult::Semicolon => return self.error_at_position(ErrorType::IllegalSemicolon),
            };
            if !found {
                if self.seek(Some('}'))? != SeekResult::Colon {
                    return self.error_at_position(ErrorType::ExpectedColon)
                }
            };
            match self.parse_expression_tail(Some('}'), vec![], ExpressionParseState::Seek)? {
                ExpressionParse::ExpressionThenDelimiter { expression } => entries.push(ParsedEntry { key, value: expression }),
                ExpressionParse::ExpressionThenClosing { expression } => {
                    entries.push(ParsedEntry { key, value: expression });
                    break;
                }
            };
        };
        let to = self.position();
        Ok(ParsedDictionary { entries, from, to })
    }

    //// Grouping

    fn parse_closed_grouping_tail_in_state(&mut self, head: Vec<ParsedArgument>, from: Position, parse_state: ExpressionParseState) -> Result<ParsedGrouping, ParseError> {
        match self.parse_expression_tail(Some('}'), head, parse_state)? {
            ExpressionParse::ExpressionThenDelimiter { .. } => self.error_at_position(ErrorType::IllegalSemicolon),
            ExpressionParse::ExpressionThenClosing { expression } => Ok(ParsedGrouping { expression, from, to: self.position() }),
        }
    }

    //// Function

    fn parse_closed_function(&mut self) -> Result<ParsedFunction, ParseError> {
        let from = self.position();
        self.parse_function_expression(Some(')'), from)
    }

    /// Parse a function expression.
    ///
    /// Assumes that the iterator is at the opening parenthesis.
    fn parse_function_expression(&mut self, expected_closing: Option<char>, from: Position) -> Result<ParsedFunction, ParseError> {
        /// It is not known immediately if an argument is positional, a flag or an option. A
        /// pending value is stored here until it is determined.
        enum PendingState {
            None,
            Pending(KeyArgument),
            Key(KeyArgument),
        }
        /// The next thing that the parser must do or parse.
        enum ParseState {
            Seek, Glyphs(char), Function, Sequence, CurlyBrackets, Enclosure, Quote, Colon, Semicolon, Closing,
        };
        fn push_argument(elements: &mut Vec<ParsedFunctionArgument>, pending: KeyArgument) {
            elements.push(ParsedFunctionArgument::Positional { argument: pending.into() });
        }
        fn handle_nonkey(pending_state: PendingState, elements: &mut Vec<ParsedFunctionArgument>, argument: ParsedArgument) -> (ParseState, PendingState) {
            if let PendingState::Key(key) = pending_state {
                elements.push(ParsedFunctionArgument::Option { key: key.into(), value: argument });
            } else {
                if let PendingState::Pending(pending) = pending_state {
                    elements.push(ParsedFunctionArgument::Positional { argument: pending.into() });
                };
                elements.push(ParsedFunctionArgument::Positional { argument });
            };
            (ParseState::Seek, PendingState::None)
        }
        fn handle_keyable(pending_state: PendingState, elements: &mut Vec<ParsedFunctionArgument>, argument: KeyArgument) -> (ParseState, PendingState) {
            match pending_state {
                PendingState::None => (ParseState::Seek, PendingState::Pending(argument)),
                PendingState::Pending(pending) => {
                    elements.push(ParsedFunctionArgument::Positional { argument: pending.into() });
                    (ParseState::Seek, PendingState::Pending(argument.into()))
                }
                PendingState::Key(key) => {
                    elements.push(ParsedFunctionArgument::Option { key: key.into(), value: argument.into() });
                    (ParseState::Seek, PendingState::None)
                }
            }
        }
        let mut elements = vec![];
        let mut parse_state = ParseState::Seek;
        let mut pending_state = PendingState::None;
        loop {
            let (a, b) = match parse_state {
                ParseState::Seek => {
                    match self.seek(expected_closing)? {
                        SeekResult::Closing => (ParseState::Closing, pending_state),
                        SeekResult::FunctionOpening => (ParseState::Function, pending_state),
                        SeekResult::SequenceOpening => (ParseState::Sequence, pending_state),
                        SeekResult::CurlyBracketOpening => (ParseState::CurlyBrackets, pending_state),
                        SeekResult::QuoteOpening => (ParseState::Quote, pending_state),
                        SeekResult::EnclosureOpening => (ParseState::Enclosure, pending_state),
                        SeekResult::Colon => (ParseState::Colon, pending_state),
                        SeekResult::Semicolon => (ParseState::Semicolon, pending_state),
                        SeekResult::Glyphs(initial) => (ParseState::Glyphs(initial), pending_state),
                    }
                }
                ParseState::Glyphs(initial) => {
                    match self.parse_glyphs(initial, expected_closing)? {
                        GlyphsParse::Symbol { symbol: glyphs, closing } => {
                            let imminent_parse_state = match closing {
                                GlyphsClosing::Whitespace => ParseState::Seek,
                                GlyphsClosing::Closing => ParseState::Closing,
                                GlyphsClosing::FunctionOpening => ParseState::Function,
                                GlyphsClosing::SequenceOpening => ParseState::Sequence,
                                GlyphsClosing::CurlyBracketOpening => ParseState::CurlyBrackets,
                                GlyphsClosing::QuoteOpening => ParseState::Quote,
                                GlyphsClosing::EnclosureOpening => ParseState::Enclosure,
                                GlyphsClosing::Colon => ParseState::Colon,
                                GlyphsClosing::Semicolon => ParseState::Semicolon,
                            };
                            if let PendingState::Key(key) = pending_state {
                                elements.push(ParsedFunctionArgument::Option { key: key.into(), value: glyphs.into() });
                                (imminent_parse_state, PendingState::None)
                            } else {
                                if let PendingState::Pending(pending) = pending_state {
                                    push_argument(&mut elements, pending);
                                };
                                let pending_glyphs = PendingState::Pending(glyphs.into());
                                (imminent_parse_state, pending_glyphs)
                            }
                        }
                        GlyphsParse::EmptyGrouping { closing, position } => {
                            let imminent_parse_state = match closing {
                                GlyphsClosing::Whitespace => ParseState::Seek,
                                GlyphsClosing::Closing => ParseState::Closing,
                                GlyphsClosing::FunctionOpening => ParseState::Function,
                                GlyphsClosing::SequenceOpening => ParseState::Sequence,
                                GlyphsClosing::CurlyBracketOpening => ParseState::CurlyBrackets,
                                GlyphsClosing::QuoteOpening => ParseState::Quote,
                                GlyphsClosing::EnclosureOpening => ParseState::Enclosure,
                                GlyphsClosing::Colon => ParseState::Colon,
                                GlyphsClosing::Semicolon => ParseState::Semicolon,
                            };
                            if let PendingState::Key(key) = pending_state {
                                elements.push(ParsedFunctionArgument::Option { key: key.into(), value: ParsedArgument::Grouping(ParsedGrouping { expression: ParsedExpression::empty(), from: position, to: position }) });
                                (imminent_parse_state, PendingState::None)
                            } else {
                                if let PendingState::Pending(pending) = pending_state {
                                    elements.push(ParsedFunctionArgument::Positional { argument: pending.into() });
                                };
                                elements.push(ParsedFunctionArgument::Positional { argument: ParsedArgument::Grouping(ParsedGrouping { expression: ParsedExpression::empty(), from: position, to: position }) });
                                (imminent_parse_state, PendingState::None)
                            }
                        }
                        GlyphsParse::Comment => {
                            (ParseState::Seek, pending_state)
                        }
                    }
                }
                ParseState::Function => {
                    let from = self.position();
                    let function = self.parse_function_expression(Some(')'), from)?;
                    handle_nonkey(pending_state, &mut elements, function.into())
                }
                ParseState::Sequence => {
                    let sequence = self.parse_sequence(Some(']'))?;
                    handle_nonkey(pending_state, &mut elements, sequence.into())
                }
                ParseState::CurlyBrackets => {
                    let curly_bracket = self.parse_closed_curly_bracket()?;
                    let argument = match curly_bracket {
                        CurlyBracketsParse::Dictionary(d) => d.into(),
                        CurlyBracketsParse::Grouping(g) => g.into(),
                    };
                    handle_nonkey(pending_state, &mut elements, argument)
                }
                ParseState::Enclosure => {
                    let symbol = self.parse_enclosed_symbol()?;
                    handle_keyable(pending_state, &mut elements, KeyArgument::Symbol(symbol))
                }
                ParseState::Quote => {
                    let quote = self.parse_double_quote()?;
                    handle_keyable(pending_state, &mut elements, KeyArgument::Quote(quote))
                }
                ParseState::Colon => {
                    match pending_state {
                        PendingState::None => return self.error_at_position(ErrorType::IllegalColon),
                        PendingState::Pending(pending) => (ParseState::Seek, PendingState::Key(pending)),
                        PendingState::Key(..) => return self.error_at_position(ErrorType::IllegalColon),
                    }
                }
                ParseState::Semicolon => {
                    match pending_state {
                        PendingState::None => return self.error_at_position(ErrorType::IllegalSemicolon),
                        PendingState::Pending(pending) => {
                            elements.push(ParsedFunctionArgument::Flag { flag: pending.into() });
                            (ParseState::Seek, PendingState::None)
                        }
                        PendingState::Key(..) => return self.error_at_position(ErrorType::IllegalSemicolon),
                    }
                }
                ParseState::Closing => {
                    match pending_state {
                        PendingState::None => { }
                        PendingState::Pending(pending) => {
                            push_argument(&mut elements, pending);
                        }
                        PendingState::Key(_) => return self.error_at_position(ErrorType::OptionNotFinished),
                    };
                    let to = self.position();
                    return Ok(ParsedFunction { arguments: elements, from, to })
                },
            };
            parse_state = a;
            pending_state = b;
        };
    }

    //// Utility

    /// Raise an error at the current position.
    fn error_at_position<T>(&mut self, error_type: ErrorType) -> Result<T, ParseError> {
        let position = self.position();
        Err(ParseError { position, error: error_type })
    }

    /// Parse enclosed characters.
    ///
    /// Parses a string until reaching the specified character. Reserved characters are also parsed
    /// as part of the string.
    ///
    /// Assumes that the iterator is at the opening character.
    fn parse_enclosed_characters(&mut self, closing_character: char, from: Position) -> Result<ParsedQuote, ParseError> {
        let mut string = String::new();
        loop {
            if let Some(c) = self.next() {
                if c == closing_character {
                    let to = self.position();
                    return Ok(ParsedQuote { quote: string, from, to });
                } else if c == '\\' {
                    if let Some(d) = self.next() {
                        if d != closing_character {
                            string.push(c);
                        };
                        string.push(d);
                    } else {
                        return self.error_at_position(ErrorType::EscapingEndOfStream);
                    };
                } else {
                    string.push(c);
                };
            } else {
                return Err(ParseError { position: self.position(), error: ErrorType::ClosingMismatch });
            };
        };
    }

    /// Seek the next glyph or reserved character.
    fn seek(&mut self, expected_closing: Option<char>) -> Result<SeekResult, ParseError> {
        loop {
            if let Some(c) = self.next() {
                if c.is_whitespace() {
                    continue;
                } else if c == ':' {
                    return Ok(SeekResult::Colon);
                } else if c == ';' {
                    return Ok(SeekResult::Semicolon);
                } else if c == ')' || c == ']' || c == '}' || c == '⟩' {
                    return if Some(c) == expected_closing {
                        Ok(SeekResult::Closing)
                    } else {
                        self.error_at_position(ErrorType::ClosingMismatch)
                    };
                } else if c == '(' {
                    return Ok(SeekResult::FunctionOpening);
                } else if c == '[' {
                    return Ok(SeekResult::SequenceOpening);
                } else if c == '{' {
                    return Ok(SeekResult::CurlyBracketOpening);
                } else if c == '⟨' {
                    return Ok(SeekResult::EnclosureOpening);
                } else if c == '"' {
                    return Ok(SeekResult::QuoteOpening);
                } else {
                    return Ok(SeekResult::Glyphs(c));
                };
            } else {
                return if expected_closing == None {
                    Ok(SeekResult::Closing)
                } else {
                    self.error_at_position(ErrorType::ClosingMismatch)
                };
            }
        }
    }

    /// Action to take and result to return when reaching a closing bracket.
    fn result_at_closing_bracket<T>(&mut self, found_bracket: char, expected_closing: Option<char>, ok_value: T) -> Result<T, ParseError> {
        if Some(found_bracket) == expected_closing {
            Ok(ok_value)
        } else {
            let position = self.position();
            Err(ParseError { position, error: ErrorType::ClosingMismatch })
        }
    }

    /// Action to take and result to return when reaching EOS.
    fn result_at_end_of_stream<T>(&mut self, expected_closing: Option<char>, ok_value: T) -> Result<T, ParseError> {
        if None == expected_closing {
            Ok(ok_value)
        } else {
            let position = self.position();
            Err(ParseError { position, error: ErrorType::ClosingMismatch })
        }
    }

}
