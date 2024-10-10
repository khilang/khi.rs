//! Parsed document model (AST) reference implementation.

use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::Iter;
use crate::{Attribute, AttributeValue, Compound, Dictionary, Element, List, Tagged, Text, Tuple, Value};

//// Position

/// A char position.
///
/// Contains a line number and a column number, corresponding to a character in
/// a document.
#[derive(PartialEq, Eq, Copy, Clone)]
pub struct Position { pub index: usize, pub line: usize, pub column: usize }

//// Value

/// A parsed value.
#[derive(Clone)]
pub enum ParsedValue {
    Text(ParsedText, Position, Position),
    Tagged(ParsedTaggedValue, Position, Position),
    Tuple(ParsedTuple, Position, Position),
    Dictionary(ParsedDictionary, Position, Position),
    List(ParsedList, Position, Position),
    Compound(ParsedCompound, Position, Position),
    Nil(Position, Position),
}

impl ParsedValue {
    pub fn nil(from: Position, to: Position) -> Self {
        ParsedValue::Nil(from, to)
    }

    pub fn from_terms(from: Position, to: Position, mut terms: Vec<ParsedValue>, whitespace: Vec<bool>) -> Self {
        let len = terms.len();
        if len == 0 {
            ParsedValue::Nil(from, to)
        } else if len == 1 {
            terms.pop().unwrap()
        } else {
            ParsedValue::Compound(ParsedCompound { components: terms, whitespace }, from, to)
        }
    }

    pub fn from(&self) -> Position {
        match self {
            ParsedValue::Nil(.., from, _) => from,
            ParsedValue::Text(.., from, _) => from,
            ParsedValue::Dictionary(.., from, _) => from,
            ParsedValue::List(.., from, _) => from,
            ParsedValue::Compound(.., from, _) => from,
            ParsedValue::Tuple(.., from, _) => from,
            ParsedValue::Tagged(.., from, _) => from,
        }.clone()
    }

    pub fn to(&self) -> Position {
        match self {
            ParsedValue::Nil(.., to) => to,
            ParsedValue::Text(.., to) => to,
            ParsedValue::Dictionary(.., to) => to,
            ParsedValue::List(.., to) => to,
            ParsedValue::Compound(.., to) => to,
            ParsedValue::Tuple(.., to) => to,
            ParsedValue::Tagged(.., to) => to,
        }.clone()
    }

    pub fn from_tuple(mut values: Vec<ParsedValue>, from: Position, to: Position) -> Self {
        let len = values.len();
        if len == 0 {
            ParsedValue::Tuple(ParsedTuple::Unit, from, to)
        } else if len == 1 {
            let value = values.remove(0);
            if matches!(value, ParsedValue::Tuple(..)) {
                ParsedValue::Tuple(ParsedTuple::Single(Box::new(value)), from, to)
            } else {
                value
            }
        } else {
            ParsedValue::Tuple(ParsedTuple::Multiple(values.into_boxed_slice()), from, to)
        }
    }

    pub fn is_unit(&self) -> bool {
        matches!(self, ParsedValue::Tuple(ParsedTuple::Unit, ..))
    }

    fn elements_as_tuple(&self) -> Vec<&ParsedValue> {
        match self {
            ParsedValue::Tuple(t, _, _) => {
                match t {
                    ParsedTuple::Unit => vec![],
                    ParsedTuple::Single(v) => vec![v],
                    ParsedTuple::Multiple(vs) => {
                        let mut r = vec![];
                        for v in vs.iter() {
                            r.push(v);
                        }
                        r
                    },
                }
            }
            v => vec![v],
        }
    }

}

impl Value<ParsedValue, ParsedText, ParsedDictionary, ParsedList, ParsedCompound, ParsedTuple, ParsedTaggedValue> for ParsedValue {
    fn is_text(&self) -> bool {
        matches!(self, ParsedValue::Text(..))
    }

    fn is_tagged(&self) -> bool {
        matches!(self, ParsedValue::Tagged(..))
    }

    fn is_tuple(&self) -> bool {
        matches!(self, ParsedValue::Tuple(..))
    }

    fn is_dictionary(&self) -> bool {
        matches!(self, ParsedValue::Dictionary(..))
    }

    fn is_list(&self) -> bool {
        matches!(self, ParsedValue::List(..))
    }

    fn is_compound(&self) -> bool {
        matches!(self, ParsedValue::Compound(..))
    }

    fn is_nil(&self) -> bool {
        matches!(self, ParsedValue::Nil(..))
    }

    fn as_text(&self) -> Option<&ParsedText> {
        if let ParsedValue::Text(t, ..) = self {
            Some(t)
        } else {
            None
        }
    }

    fn as_tagged(&self) -> Option<&ParsedTaggedValue> {
        if let ParsedValue::Tagged(d, ..) = self {
            Some(d)
        } else {
            None
        }
    }

    fn as_tuple(&self) -> Option<&ParsedTuple> {
        if let ParsedValue::Tuple(d, _, _) = self {
            Some(d)
        } else {
            None
        }
    }

    fn as_dictionary(&self) -> Option<&ParsedDictionary> {
        if let ParsedValue::Dictionary(d, ..) = self {
            Some(d)
        } else {
            None
        }
    }

    fn as_list(&self) -> Option<&ParsedList> {
        if let ParsedValue::List(t, ..) = self {
            Some(t)
        } else {
            None
        }
    }

    fn as_compound(&self) -> Option<&ParsedCompound> {
        if let ParsedValue::Compound(d, ..) = self {
            Some(d)
        } else {
            None
        }
    }

    fn as_mut_text(&mut self) -> Option<&mut ParsedText> {
        if let ParsedValue::Text(t, ..) = self {
            Some(t)
        } else {
            None
        }
    }

    fn as_mut_tagged(&mut self) -> Option<&mut ParsedTaggedValue> {
        if let ParsedValue::Tagged(d, ..) = self {
            Some(d)
        } else {
            None
        }
    }

    fn as_mut_tuple(&mut self) -> Option<&mut ParsedTuple> {
        if let ParsedValue::Tuple(d, _, _) = self {
            Some(d)
        } else {
            None
        }
    }

    fn as_mut_dictionary(&mut self) -> Option<&mut ParsedDictionary> {
        if let ParsedValue::Dictionary(d, ..) = self {
            Some(d)
        } else {
            None
        }
    }

    fn as_mut_list(&mut self) -> Option<&mut ParsedList> {
        if let ParsedValue::List(t, ..) = self {
            Some(t)
        } else {
            None
        }
    }

    fn as_mut_compound(&mut self) -> Option<&mut ParsedCompound> {
        if let ParsedValue::Compound(d, ..) = self {
            Some(d)
        } else {
            None
        }
    }

    fn iter_as_tuple<'b> (&'b self) -> impl Iterator<Item=&'b ParsedValue> where Self: 'b {
        if let ParsedValue::Tuple(tuple, ..) = self {
            tuple.iter()
        } else {
            TupleIterator::Single(false, self)
        }
    }

    fn len_as_tuple(&self) -> usize {
        match self {
            ParsedValue::Tuple(t, _, _) => t.len(),
            _ => 1,
        }
    }

}

//// Text

#[derive(PartialEq, Eq, Clone)]
pub struct ParsedText {
    pub str: Rc<str>
}

impl Text<ParsedValue, ParsedText, ParsedDictionary, ParsedList, ParsedCompound, ParsedTuple, ParsedTaggedValue> for ParsedText {
    fn as_str(&self) -> &str {
        &self.str
    }
}

//// Tagged value

/// A parsed tagged value.
#[derive(Clone)]
pub struct ParsedTaggedValue {
    pub name: Rc<str>,
    pub attributes: Vec<ParsedAttribute>,
    pub value: Box<ParsedValue>,
}

impl Tagged<ParsedValue, ParsedText, ParsedDictionary, ParsedList, ParsedCompound, ParsedTuple, Self> for ParsedTaggedValue {
    type AttributeIterator<'b> = AttributeIterator<'b>;

    fn name(&self) -> &str {
        &self.name
    }

    fn has_attributes(&self) -> bool {
        !self.attributes.is_empty()
    }

    fn get_attribute_by(&self, key: &str) -> Option<AttributeValue<'_>> {
        for attribute in &self.attributes {
            if key.eq(attribute.key().deref()) {
                return match attribute {
                    ParsedAttribute(_, Some(v)) => Some(AttributeValue(Some(&v))),
                    ParsedAttribute(_, None) => Some(AttributeValue(None)),
                };
            }
        }
        None
    }

    fn get_attribute_at(&self, index: usize) -> Option<Attribute<'_>> {
        if let Some(attribute) = self.attributes.get(index) {
            match attribute {
                ParsedAttribute(k, Some(v)) => Some(Attribute(&k, Some(&v))),
                ParsedAttribute(k, None) => Some(Attribute(&k, None)),
            }
        } else {
            None
        }
    }

    fn iter_attributes(&self) -> Self::AttributeIterator<'_> {
        AttributeIterator { iter: self.attributes.iter() }
    }

    fn get(&self) -> &ParsedValue {
        &self.value
    }
}

#[derive(Clone)]
pub struct ParsedAttribute(pub Rc<str>, pub Option<Rc<str>>);

impl ParsedAttribute {
    fn key(&self) -> Rc<str> {
        self.0.clone()
    }
}

pub struct AttributeIterator<'a> {
    iter: Iter<'a, ParsedAttribute>,
}

impl<'a> Iterator for AttributeIterator<'a> {
    type Item = Attribute<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ParsedAttribute(key, value)) = self.iter.next() {
            if let Some(value) = value {
                Some(Attribute(key, Some(value)))
            } else {
                Some(Attribute(key, None))
            }
        } else {
            None
        }
    }
}

//// Tuple

/// A parsed tuple.
#[derive(Clone)]
pub enum ParsedTuple {
    Unit,
    Single(Box<ParsedValue>), // Value must be a ParsedTuple
    Multiple(Box<[ParsedValue]>),
}

impl Tuple<ParsedValue, ParsedText, ParsedDictionary, ParsedList, ParsedCompound, Self, ParsedTaggedValue> for ParsedTuple {
    type TupleIterator<'b> = TupleIterator<'b>;

    fn len(&self) -> usize {
        match self {
            ParsedTuple::Unit => 0,
            ParsedTuple::Single(_) => 1,
            ParsedTuple::Multiple(m) => m.len(),
        }
    }

    fn is_empty(&self) -> bool {
        matches!(self, ParsedTuple::Unit)
    }

    fn get(&self, index: usize) -> Option<&ParsedValue> {
        match self {
            ParsedTuple::Unit => None,
            ParsedTuple::Single(v) => {
                Some(v.as_ref())
            }
            ParsedTuple::Multiple(m) => {
                m.get(index)
            }
        }
    }

    fn iter(&self) -> Self::TupleIterator<'_> {
        match self {
            ParsedTuple::Unit => TupleIterator::Unit,
            ParsedTuple::Single(v) => TupleIterator::Single(false, v),
            ParsedTuple::Multiple(m) => TupleIterator::Multiple(0, m.as_ref()),
        }
    }
}

pub enum TupleIterator<'a> {
    Unit,
    Single(bool, &'a ParsedValue),
    Multiple(usize, &'a [ParsedValue]),
}

impl<'a> Iterator for TupleIterator<'a> {
    type Item = &'a ParsedValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TupleIterator::Unit => None,
            TupleIterator::Single(done, v) => {
                if *done {
                    None
                } else {
                    *done = true;
                    Some(v)
                }
            }
            TupleIterator::Multiple(index, v) => {
                let v = v.get(*index);
                *index += 1;
                v
            }
        }
    }
}

//// Dictionary

/// A parsed dictionary.
#[derive(Clone)]
pub struct ParsedDictionary {
    pub entries: HashMap<Rc<str>, ParsedValue>,
}

impl ParsedDictionary {
    pub fn empty() -> Self {
        ParsedDictionary { entries: HashMap::new() }
    }
}

impl Dictionary<ParsedValue, ParsedText, ParsedDictionary, ParsedList, ParsedCompound, ParsedTuple, ParsedTaggedValue> for ParsedDictionary {
    type EntryIterator<'b> = EntryIterator<'b>;

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn get(&self, key: &str) -> Option<&ParsedValue> {
        if let Some(entry) = self.entries.get(key) {
            Some(entry)
        } else {
            None
        }
    }

    fn get_mut(&mut self, key: &str) -> Option<&mut ParsedValue> {
        if let Some(entry) = self.entries.get_mut(key) {
            Some(entry)
        } else {
            None
        }
    }

    fn iter(&self) -> Self::EntryIterator<'_> {
        EntryIterator(self.entries.iter())
    }
}

pub struct EntryIterator<'a>(std::collections::hash_map::Iter<'a, Rc<str>, ParsedValue>);

impl<'a> Iterator for EntryIterator<'a> {
    type Item = (&'a str, &'a ParsedValue);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((s, v)) = self.0.next() {
            Some((s.as_ref(), v))
        } else {
            None
        }
    }
}

//// List

/// A parsed list.
#[derive(Clone)]
pub struct ParsedList {
    pub elements: Vec<ParsedValue>,
}

impl ParsedList {
    /// Empty list.
    pub fn empty() -> Self {
        ParsedList { elements: vec![] }
    }
}

impl List<ParsedValue, ParsedText, ParsedDictionary, Self, ParsedCompound, ParsedTuple, ParsedTaggedValue> for ParsedList {
    type ListIterator<'b> = Iter<'b, ParsedValue>;

    fn len(&self) -> usize {
        self.elements.len()
    }

    fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    fn get_element(&self, index: usize) -> Option<&ParsedValue> {
        self.elements.get(index)
    }

    fn iter(&self) -> Self::ListIterator<'_> {
        self.elements.iter()
    }
}

//// Compound

#[derive(Clone)]
pub struct ParsedCompound {
    pub components: Vec<ParsedValue>, // Todo reorganize
    pub whitespace: Vec<bool>,
}

impl ParsedCompound {}

impl Compound<ParsedValue, ParsedText, ParsedDictionary, ParsedList, Self, ParsedTuple, ParsedTaggedValue> for ParsedCompound {
    type ElementIterator<'b> = ElementIterator<'b>;

    fn len(&self) -> usize {
        self.components.len() + self.whitespace.len()
    }

    fn get(&self, _index: usize) -> Option<Element<&ParsedValue>> {
        todo!()
    }

    fn iter(&self) -> Self::ElementIterator<'_> {
        ElementIterator {
            components: &self.components,
            whitespace: &self.whitespace,
            index: 0,
            after_component: false,
        }
    }
}

pub struct ElementIterator<'b> {
    components: &'b [ParsedValue],
    whitespace: &'b [bool],
    index: usize,
    after_component: bool,
}

impl<'b> Iterator for ElementIterator<'b> {
    type Item = Element<&'b ParsedValue>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index != self.components.len() - 1 {
            if self.after_component {
                let whitespace = self.whitespace[self.index];
                self.index += 1;
                if whitespace {
                    self.after_component = false;
                    return Some(Element::Whitespace);
                };
            };
        } else {
            if self.after_component {
                return None;
            };
        };
        self.after_component = true;
        Some(Element::Element(&self.components[self.index]))
    }
}
