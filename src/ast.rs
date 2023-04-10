//! Structure produced by the parser.


use std::fmt::{Display, Formatter};
use crate::lex::Position;


//// Parsing results
////
//// Parsing a document yields a nested structure consisting of these structures.


#[derive(PartialEq, Eq, Clone)]
pub enum ParsedExpression {
    Empty(Position, Position),
    Text(ParsedText),
    Sequence(ParsedSequence),
    Dictionary(ParsedDictionary),
    Command(ParsedCommand),
    Compound(ParsedCompound),
}


pub enum ExpressionIter<'a> {
    Empty,
    Argument { argument: &'a ParsedArgument, done: bool },
    Compound { arguments: &'a Vec<(ParsedArgument, bool)>, index: usize },
}


impl <'a> Iterator for ExpressionIter<'a> {

    type Item = (&'a ParsedArgument, bool);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ExpressionIter::Empty => None,
            ExpressionIter::Argument { argument, done } => {
                if !*done {
                    *done = true;
                    Some((argument, false))
                } else {
                    None
                }
            }
            ExpressionIter::Compound { arguments, index } => {
                if *index < arguments.len() {
                    let &(a, b) = &arguments.get(*index).unwrap();
                    *index += 1;
                    Some((&a, *b))
                } else {
                    None
                }
            }
        }
    }

}


pub type ParsedArgument = ParsedExpression;


impl ParsedExpression {

    pub fn empty(from: Position, to: Position) -> Self {
        ParsedExpression::Empty(from ,to)
    }

    pub fn length(&self) -> usize {
        match self {
            ParsedArgument::Empty(..) => 0,
            ParsedArgument::Text(ParsedText { .. }) => 1,
            ParsedArgument::Sequence(ParsedSequence { .. }) => 1,
            ParsedArgument::Dictionary(ParsedDictionary { .. }) => 1,
            ParsedArgument::Command(ParsedCommand { .. }) => 1,
            ParsedArgument::Compound(ParsedCompound { arguments, .. }) => arguments.len(),
        }.clone()
    }

    pub fn is_empty(&self) -> bool {
        self.length() == 0
    }

    pub fn iter(&self) -> ExpressionIter {
        match self {
            ParsedExpression::Empty(..) => ExpressionIter::Empty,
            ParsedExpression::Text(..) | ParsedExpression::Sequence(..) | ParsedExpression::Dictionary(..) | ParsedExpression::Command(..) => {
                ExpressionIter::Argument { argument: &self, done: false }
            },
            ParsedExpression::Compound(c) => ExpressionIter::Compound { arguments: &c.arguments, index: 0 },
        }
    }

}


impl ParsedArgument {

    pub fn from(&self) -> Position {
        match self {
            ParsedArgument::Empty(from, ..) => from,
            ParsedArgument::Text(ParsedText { from, .. }) => from,
            ParsedArgument::Sequence(ParsedSequence { from, .. }) => from,
            ParsedArgument::Dictionary(ParsedDictionary { from, .. }) => from,
            ParsedArgument::Command(ParsedCommand { from, .. }) => from,
            ParsedArgument::Compound(ParsedCompound { from, .. }) => from,
        }.clone()
    }

    pub fn to(&self) -> Position {
        match self {
            ParsedArgument::Empty(.., to) => to,
            ParsedArgument::Text(ParsedText { to, .. }) => to,
            ParsedArgument::Sequence(ParsedSequence { to, .. }) => to,
            ParsedArgument::Dictionary(ParsedDictionary { to, .. }) => to,
            ParsedArgument::Command(ParsedCommand { to, .. }) => to,
            ParsedArgument::Compound(ParsedCompound { to, .. }) => to,
        }.clone()
    }

    pub fn is_text(&self) -> bool {
        if let ParsedArgument::Text(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_sequence(&self) -> bool {
        if let ParsedArgument::Sequence(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_dictionary(&self) -> bool {
        if let ParsedArgument::Dictionary(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_command(&self) -> bool {
        if let ParsedArgument::Command(..) = self {
            true
        } else {
            false
        }
    }

    pub fn is_compound(&self) -> bool {
        if let ParsedArgument::Compound(..) = self {
            true
        } else {
            false
        }
    }

}


impl From<ParsedText> for ParsedArgument {

    fn from(text: ParsedText) -> Self {
        ParsedArgument::Text(text)
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


impl From<ParsedCommand> for ParsedArgument {

    fn from(command: ParsedCommand) -> Self {
        ParsedArgument::Command(command)
    }

}


impl From<ParsedCompound> for ParsedArgument {

    fn from(compound: ParsedCompound) -> Self {
        ParsedArgument::Compound(compound)
    }

}


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedText { pub text: String, pub from: Position, pub to: Position }


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedSequence { pub elements: Vec<ParsedExpression>, pub from: Position, pub to: Position }


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedEntry { pub key: ParsedText, pub value: ParsedExpression }


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedDictionary { pub entries: Vec<ParsedEntry>, pub from: Position, pub to: Position }


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedCompound { pub arguments: Vec<(ParsedExpression, bool)>, pub from: Position, pub to: Position }


pub type ParsedAttribute = ParsedEntry;


#[derive(PartialEq, Eq, Clone)]
pub struct ParsedCommand {
    pub command: String,
    pub attributes: Vec<ParsedAttribute>,
    pub arguments: Vec<ParsedExpression>,
    pub from: Position,
    pub to: Position,
}


impl ParsedDictionary {

    pub fn empty(from: Position, to: Position) -> Self {
        ParsedDictionary { entries: vec![], from, to }
    }

    pub fn get_entry(&self, key: &str) -> Option<&ParsedEntry> {
        for entry in &self.entries {
            if entry.key.text.eq(key) {
                return Some(&entry);
            };
        }
        return None;
    }

    pub fn get(&self, key: &str) -> Option<&ParsedExpression> {
        for entry in &self.entries {
            if entry.key.text.eq(key) {
                return Some(&entry.value);
            };
        }
        return None;
    }

}


impl ParsedCompound {

    pub fn new(expression: Vec<(ParsedExpression, bool)>, from: Position, to: Position ) -> Self {
        ParsedCompound { arguments: expression, from, to }
    }

}


//// Parsing result formatting


/// Check if a string has reserved characters. Such a string must be escaped.
pub fn has_reserved(string: &str) -> bool {
    string.contains(|c: char| {
        c.is_whitespace() || c == ':' || c == ';' || c == '(' || c == '[' || c == '{' || c == '⟨' ||
            c == ')' || c == ']' || c == '}' || c == '⟩' || c == '"' || c == '\\'
    })
}


// impl Display for ParsedExpression {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let len = self.arguments.len();
//         if len > 0 {
//             write!(f, "{}", self.arguments.get(0).unwrap())?;
//         };
//         let mut i = 1;
//         while i < self.arguments.len() {
//             write!(f, " {}", self.arguments.get(i).unwrap())?;
//             i += 1;
//         };
//         Ok(())
//     }
// }


impl Display for ParsedExpression {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsedExpression::Empty(_, _) => write!(f, "{{}}"),
            ParsedExpression::Text(s) => s.fmt(f),
            ParsedExpression::Sequence(s) => s.fmt(f),
            ParsedExpression::Dictionary(d) => d.fmt(f),
            ParsedExpression::Command(c) => c.fmt(f),
            ParsedExpression::Compound(c) => c.fmt(f),
        }
    }

}


impl Display for ParsedText {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let reserved = has_reserved(&self.text);
        if reserved {
            write!(f, "⟨{}⟩", self.text)
        } else {
            write!(f, "{}", self.text)
        }
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
        write!(f, ":")?;
        self.value.fmt(f)?;
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


impl Display for ParsedCompound {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut iter = self.arguments.iter();
        let mut last_whitespace = true;
        loop {
            if let Some((argument, whitespace)) = iter.next() {
                if last_whitespace {
                    write!(f, " ")?;
                };
                last_whitespace = *whitespace;
                argument.fmt(f)?;
            } else {
                break;
            };
        }
        write!(f, "}}")?;
        Ok(())
    }

}


impl Display for ParsedCommand {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Command opening and name
        write!(f, "({}", &self.command)?;
        // Command attributes
        let len = self.attributes.len();
        if len > 0 {
            write!(f, " ")?;
            self.attributes.first().unwrap().fmt(f)?;
        };
        let mut i = 1;
        while i < len {
            let a = self.attributes.get(i).unwrap();
            write!(f, " ")?;
            a.fmt(f)?;
            i += 1;
        }
        // Command closing
        write!(f, ")")?;
        // Command arguments
        let mut arguments = self.arguments.iter();
        loop {
            if let Some(argument) = arguments.next() {
                write!(f, ":{{")?;
                argument.fmt(f)?;
                write!(f, "}}")?;
            } else {
                write!(f, " ")?;
                break;
            };
        };
        Ok(())
    }

}
