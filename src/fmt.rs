//! Khi structure formatter. Writes Khi structures to strings.

use std::fmt::Display;
use crate::{Component, Directive, Expression, Table, Text};

pub fn fast_format_expression<
    Ex: Expression<Tx, Dc, Tb, Dr, Cm>,
    Tx: Text<Ex, Dc, Tb, Dr, Cm>,
    Dc: Dictionry<Ex, Tx, Tb, Dr, Cm>,
    Tb: Table<Ex, Tx, Dc, Dr, Cm>,
    Dr: Directive<Ex, Tx, Dc, Tb, Cm>,
    Cm: Component<Ex, Tx, Dc, Tb, Dr>,
>(expression: Ex) {
    let len = expression.length();
    if len == 0 {

    } else if len == 1 {

    } else {

    }
}

pub fn fast_format_table<Ta: Table>(table: Ta) {

}

pub fn fast_format_dictionary() {

}

pub trait FastFormatter {
    fn format_compact(&self) -> String;
}

impl <St: Expression<_, _, _, _, _>> FastFormatter for St {

    fn format_compact(&self) -> String {
        todo!()
    }

}

pub fn pretty_format_structure() {

}

pub fn pretty_format_table() {

}

pub fn pretty_format_dictionary() {

}

pub trait PrettyFormatter {

}

impl <St: Expression<_, _, _, _, _>> PrettyFormatter for St {
    fn g(&self) {
        self.as
    }
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

impl Display for ParsedComponent {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsedComponent::Empty(_, _) => write!(f, "{{}}"),
            ParsedComponent::Text(s) => s.fmt(f),
            ParsedComponent::Table(s) => s.fmt(f),
            ParsedComponent::Dictionary(d) => d.fmt(f),
            ParsedComponent::Directive(c) => c.fmt(f),
            ParsedComponent::Compound(c, _, _) => c.fmt(f),
        }
    }

}

//// Display

impl Display for ParsedText {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let reserved = has_reserved(&self.str);
        if reserved {
            write!(f, "⟨{}⟩", self.str)
        } else {
            write!(f, "{}", self.str)
        }
    }

}

impl Display for ParsedTable {

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


impl Display for ParsedDirective {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Command opening and name
        write!(f, "<{}", &self.directive)?;
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
        write!(f, ">")?;
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
