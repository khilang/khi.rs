//! Parsed document model (AST) reference implementation.

use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::{Iter, IterMut};
use crate::{Attribute, AttributeValue, Catenation, Dictionary, Element, List, ParsedTupleElement, Tagged, Text, Tuple, Value};

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
    Catenation(ParsedCatenation, Position, Position),
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
            let mut components = vec![];
            let mut i = 0;
            while i < terms.len() - 1 {
                components.push(Element::Element(terms[i].clone()));
                if whitespace[i] {
                    components.push(Element::Whitespace);
                }
                i += 1;
            }
            components.push(Element::Element(terms[i].clone())); // TODO No clone
            ParsedValue::Catenation(ParsedCatenation { components }, from, to)
        }
    }

    pub fn from(&self) -> Position {
        match self {
            ParsedValue::Nil(.., from, _) => from,
            ParsedValue::Text(.., from, _) => from,
            ParsedValue::Dictionary(.., from, _) => from,
            ParsedValue::List(.., from, _) => from,
            ParsedValue::Catenation(.., from, _) => from,
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
            ParsedValue::Catenation(.., to) => to,
            ParsedValue::Tuple(.., to) => to,
            ParsedValue::Tagged(.., to) => to,
        }.clone()
    }

    pub fn is_unit(&self) -> bool {
        if let Some(tuple) = self.as_tuple() {
            tuple.is_empty()
        } else {
            false
        }
    }

}

impl Value<ParsedValue, ParsedText, ParsedDictionary, ParsedList, ParsedCatenation, ParsedTuple, ParsedTaggedValue> for ParsedValue {
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

    fn is_catenation(&self) -> bool {
        matches!(self, ParsedValue::Catenation(..))
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

    fn as_catenation(&self) -> Option<&ParsedCatenation> {
        if let ParsedValue::Catenation(d, ..) = self {
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

    fn as_mut_catenation(&mut self) -> Option<&mut ParsedCatenation> {
        if let ParsedValue::Catenation(d, ..) = self {
            Some(d)
        } else {
            None
        }
    }

    fn iter_as_tuple<'b> (&'b self) -> impl Iterator<Item=ParsedTupleElement<'b, &'b ParsedValue>> where Self: 'b {
        if let ParsedValue::Tuple(tuple, ..) = self {
            AsTupleIter::of_tuple(tuple)
        } else {
            AsTupleIter::of_value(self)
        }
    }

    fn len_as_tuple(&self) -> usize {
        match self {
            ParsedValue::Tuple(t, _, _) => t.len(),
            _ => 1,
        }
    }

}

enum AsTupleIter<'a> {
    Value(bool, &'a ParsedValue),
    Tuple(TupleIterator<'a>),
}

impl<'a> AsTupleIter<'a> {
    fn of_value(value: &'a ParsedValue) -> Self {
        Self::Value(false, value)
    }
    fn of_tuple(tuple: &'a ParsedTuple) -> Self {
        Self::Tuple(TupleIterator(tuple.0.iter()))
    }
}

impl<'a> Iterator for AsTupleIter<'a> {
    type Item = ParsedTupleElement<'a, &'a ParsedValue>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AsTupleIter::Value(done, value) => {
                if *done {
                    None
                } else {
                    *done = true;
                    Some(ParsedTupleElement::Element(value))
                }
            }
            AsTupleIter::Tuple(iter) => {
                if let Some(t) = iter.next() {
                    match t {
                        ParsedTupleElement::Element(e) => {
                            Some(ParsedTupleElement::Element(e))
                        }
                        ParsedTupleElement::NamedElement(k, v) => {
                            Some(ParsedTupleElement::NamedElement(k.clone(), v))
                        }
                    }
                } else {
                    None
                }
            },
        }
    }   
}

//// Text

#[derive(PartialEq, Eq, Clone)]
pub struct ParsedText {
    pub str: Rc<str>,
    pub escapes: Vec<bool>,
}

impl Text<ParsedValue, ParsedText, ParsedDictionary, ParsedList, ParsedCatenation, ParsedTuple, ParsedTaggedValue> for ParsedText {
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
    pub tuple: ParsedTuple,
}

impl Tagged<ParsedValue, ParsedText, ParsedDictionary, ParsedList, ParsedCatenation, ParsedTuple, Self> for ParsedTaggedValue {
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
                    ParsedAttribute::Flag(_) => Some(AttributeValue::Flag),
                    ParsedAttribute::Valued(_, v) => Some(AttributeValue::Value(v.as_ref())),
                };
            }
        }
        None
    }

    fn get_attribute_at(&self, index: usize) -> Option<Attribute<'_>> {
        if let Some(attribute) = self.attributes.get(index) {
            match attribute {
                ParsedAttribute::Flag(k) => Some(Attribute(&k, None)),
                ParsedAttribute::Valued(k, v) => Some(Attribute(&k, Some(&v))),
            }
        } else {
            None
        }
    }

    fn iter_attributes(&self) -> Self::AttributeIterator<'_> {
        AttributeIterator { iter: self.attributes.iter() }
    }

    fn get(&self) -> &ParsedTuple {
        &self.tuple
    }
}

#[derive(Clone)]
pub enum ParsedAttribute {
    Flag(Rc<str>),
    Valued(Rc<str>, Rc<str>),
}

impl ParsedAttribute {
    fn key(&self) -> Rc<str> {
        match self {
            ParsedAttribute::Flag(key) | ParsedAttribute::Valued(key, ..) => key.clone(),
        }
        
    }
}

pub struct AttributeIterator<'a> {
    iter: Iter<'a, ParsedAttribute>,
}

impl<'a> Iterator for AttributeIterator<'a> {
    type Item = Attribute<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(a) = self.iter.next() {
            match a {
                ParsedAttribute::Flag(key) => {
                    Some(Attribute(key, None))
                }
                ParsedAttribute::Valued(key, value) => {
                    Some(Attribute(key, Some(value)))
                }
            }
        } else {
            None
        }
    }
}

//// Tuple

#[derive(Clone)]
pub enum TupleElement {
    Value(ParsedValue),
    Named(Rc<str>, ParsedValue),
}

/// A parsed tuple.
#[derive(Clone)]
pub struct ParsedTuple(pub(crate) Vec<TupleElement>);

impl Tuple<ParsedValue, ParsedText, ParsedDictionary, ParsedList, ParsedCatenation, Self, ParsedTaggedValue> for ParsedTuple {
    type TupleIterator<'b> = TupleIterator<'b>;
    type TupleIteratorMut<'b> = TupleIteratorMut<'b>;

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn get(&self, index: usize) -> Option<ParsedTupleElement<&ParsedValue>> {
        if let Some(e) = self.0.get(index) {
            match e {
                TupleElement::Value(v) => Some(ParsedTupleElement::Element(v)),
                TupleElement::Named(k, v) => Some(ParsedTupleElement::NamedElement(k.as_ref(), v)),
            }
        } else {
            None
        }
    }

    fn get_mut(&mut self, index: usize) -> Option<ParsedTupleElement<&mut ParsedValue>> {
        if let Some(e) = self.0.get_mut(index) {
            match e {
                TupleElement::Value(v) => Some(ParsedTupleElement::Element(v)),
                TupleElement::Named(k, v) => Some(ParsedTupleElement::NamedElement(k, v)),
            }
        } else {
            None
        }
    }

    fn iter(&self) -> Self::TupleIterator<'_> {
        TupleIterator(self.0.iter())
    }

    fn iter_mut(&mut self) -> Self::TupleIteratorMut<'_> {
        TupleIteratorMut(self.0.iter_mut())
    }
}

pub struct TupleIterator<'a>(Iter<'a, TupleElement>);

impl<'a> Iterator for TupleIterator<'a> {
    type Item = ParsedTupleElement<'a, &'a ParsedValue>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(e) = self.0.next() {
            match e {
                TupleElement::Value(v) => Some(ParsedTupleElement::Element(v)),
                TupleElement::Named(k, v) => Some(ParsedTupleElement::NamedElement(k, v)),
            }
        } else {
            None
        }
    }
}

pub struct TupleIteratorMut<'a>(IterMut<'a, TupleElement>);

impl<'a> Iterator for TupleIteratorMut<'a> {
    type Item = ParsedTupleElement<'a, &'a mut ParsedValue>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(e) = self.0.next() {
            match e {
                TupleElement::Value(v) => Some(ParsedTupleElement::Element(v)),
                TupleElement::Named(k, v) => Some(ParsedTupleElement::NamedElement(k, v)),
            }
        } else {
            None
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

impl Dictionary<ParsedValue, ParsedText, ParsedDictionary, ParsedList, ParsedCatenation, ParsedTuple, ParsedTaggedValue> for ParsedDictionary {
    type EntryIterator<'b> = EntryIterator<'b>;
    type EntryIteratorMut<'b> = EntryIteratorMut<'b>;

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

    fn iter_mut(&mut self) -> Self::EntryIteratorMut<'_> {
        EntryIteratorMut(self.entries.iter_mut())
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

pub struct EntryIteratorMut<'a>(std::collections::hash_map::IterMut<'a, Rc<str>, ParsedValue>);

impl<'a> Iterator for EntryIteratorMut<'a> {
    type Item = (&'a str, &'a mut ParsedValue);

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

impl List<ParsedValue, ParsedText, ParsedDictionary, Self, ParsedCatenation, ParsedTuple, ParsedTaggedValue> for ParsedList {
    type ListIterator<'b> = Iter<'b, ParsedValue>;
    type ListIteratorMut<'b> = IterMut<'b, ParsedValue>;

    fn len(&self) -> usize {
        self.elements.len()
    }

    fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    fn get(&self, index: usize) -> Option<&ParsedValue> {
        self.elements.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut ParsedValue> {
        self.elements.get_mut(index)
    }

    fn iter(&self) -> Self::ListIterator<'_> {
        self.elements.iter()
    }

    fn iter_mut(&mut self) -> Self::ListIteratorMut<'_> {
        self.elements.iter_mut()
    }
}

//// Catenation

#[derive(Clone)]
pub struct ParsedCatenation {
    pub components: Vec<Element<ParsedValue>>, // Todo reorganize
}

impl ParsedCatenation {}

impl Catenation<ParsedValue, ParsedText, ParsedDictionary, ParsedList, Self, ParsedTuple, ParsedTaggedValue> for ParsedCatenation {
    type ElementIterator<'b> = Iter<'b, Element<ParsedValue>>;
    type ElementIteratorMut<'b> = IterMut<'b, Element<ParsedValue>>;

    fn len(&self) -> usize {
        self.components.len()
    }

    fn get(&self, index: usize) -> Option<&Element<ParsedValue>> {
        self.components.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Element<ParsedValue>> {
        self.components.get_mut(index)
    }

    fn iter(&self) -> Self::ElementIterator<'_> {
        self.components.iter()
    }

    fn iter_mut(&mut self) -> Self::ElementIteratorMut<'_> {
        self.components.iter_mut()
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
