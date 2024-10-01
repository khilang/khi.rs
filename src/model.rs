//! Plain in-memory representation of Khi data structures.

use crate::{Dictionary, Pattern, Structure, Table};

pub enum SimpleValue {
    Nil,
    Text(SimpleText),
    Dictionary(SimpleDictionary),
    List(SimpleList),
    Compound(Vec<SimpleValue>),
    Tagged(SimpleTagged),
}

impl Value<SimpleText, SimpleDictionary, SimpleList, SimpleComponent, SimpleTagged> for SimpleValue {

    type StructureIterator = ();

    fn length(&self) -> usize {
        match self {
            SimpleValue::Text(..) => 1,
            SimpleValue::List(..) => 1,
            SimpleValue::Dictionary(..) => 1,
            SimpleValue::Tagged(..) => 1,
            SimpleValue::Compound(v) => v.len(),
            SimpleValue::Nil => 0,
        }
    }

    fn is_empty(&self) -> bool {
        matches!(self, SimpleStructure::Empty(..))
    }

    fn is_unary(&self) -> bool {
        todo!()
    }

    fn is_compound(&self) -> bool {
        todo!()
    }

    fn get(&self, index: usize) -> Option<&Component> {
        todo!()
    }

    fn iter(&self) -> Self::StructureIterator {
        todo!()
    }

    fn conform_text(&self) -> Option<&Text<Self, Table, Dictionary, Directive, Component>> {
        todo!()
    }

    fn conform_table(&self) -> Option<&Table> {
        todo!()
    }

    fn conform_dictionary(&self) -> Option<&Dictionary> {
        todo!()
    }

    fn conform_directive(&self) -> Option<&Directive> {
        todo!()
    }

    fn is_text(&self) -> bool {
        todo!()
    }

    fn is_dictionary(&self) -> bool {
        todo!()
    }

    fn is_table(&self) -> bool {
        todo!()
    }

    fn is_pattern(&self) -> bool {
        todo!()
    }

}

/// A simple text implementation.
pub struct SimpleText {
    string: String,
}

impl SimpleText {

    fn new(string: String) -> Self {
        Self { string }
    }

    fn from_str(str: &str) -> Self {
        Self { string: String::from(str) }
    }

}

/// A simple table implementation.
pub struct SimpleList {
    entries: Vec<SimpleValue>,
}

/// A simple dictionary implementation.
pub struct SimpleDictionary {
    entries: Vec<(SimpleText, SimpleValue)>,
}

/// A simple tag implementation.
pub struct SimpleTagged {
    header: String,
    attributes: Vec<(SimpleText, SimpleValue)>,
    arguments: Vec<SimpleComponent>
}
