//! Plain in-memory Khi structures.

use crate::{Dictionary, Expression, Table};

pub enum SimpleStructure {
    Empty,
    Text(SimpleText),
    Table(SimpleTable),
    Dictionary(SimpleDictionary),
    Directive(SimpleDirective),
    Compound(Vec<SimpleComponent>)
}

impl Expression<SimpleText, SimpleTable, SimpleDictionary, SimpleDirective, SimpleComponent> for SimpleStructure {

    type StructureIterator = ();

    fn length(&self) -> usize {
        match self {
            SimpleStructure::Empty => 0,
            SimpleStructure::Text(..) => 1,
            SimpleStructure::Table(..) => 1,
            SimpleStructure::Dictionary(..) => 1,
            SimpleStructure::Directive(..) => 1,
            SimpleStructure::Compound(v) => v.len(),
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

    fn is_table(&self) -> bool {
        todo!()
    }

    fn is_dictionary(&self) -> bool {
        todo!()
    }

    fn is_directive(&self) -> bool {
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
pub struct SimpleTable {
    entries: Vec<SimpleStructure>,
}

/// A simple dictionary implementation.
pub struct SimpleDictionary {
    entries: Vec<(SimpleText, SimpleStructure)>,
}

/// A simple directive implementation.
pub struct SimpleDirective {
    header: String,
    attributes: Vec<(SimpleText, SimpleStructure)>,
    arguments: Vec<SimpleComponent>
}

#[repr(transparent)]
pub struct SimpleComponent(SimpleStructure);