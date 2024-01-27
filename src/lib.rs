extern crate core;

#[cfg(feature = "parse")]
pub mod lex;
#[cfg(feature = "parse")]
pub mod parse;

//#[cfg(feature = "enc")]
//pub mod enc;

#[cfg(feature = "html")]
pub mod html;

#[cfg(feature = "tex")]
pub mod tex;

#[cfg(feature = "serde")]
pub mod ser;
#[cfg(feature = "serde")]
pub mod de;

//mod fmt;
//mod model;

/// A structure value.
///
/// Corresponds to a real data structure.
///
/// Is one of:
/// - nil
/// - text
/// - dictionary
/// - table
/// - composition
/// - pattern
pub trait Value<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Pt>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Pt>,
    Cm: Composition<Vl, Tx, Dc, Tb, Cm, Pt>,
    Pt: Pattern<Vl, Tx, Dc, Tb, Cm, Pt>,
> {
    /// Get as text.
    fn as_text(&self) -> Option<&Tx>;
    /// Get as a dictionary.
    fn as_dictionary(&self) -> Option<&Dc>;
    /// Get as a table.
    fn as_table(&self) -> Option<&Tb>;
    /// Get as a composition.
    fn as_composition(&self) -> Option<&Cm>;
    /// Get as a pattern.
    fn as_pattern(&self) -> Option<&Pt>;
    /// Check if this is nil.
    fn is_nil(&self) -> bool;
    /// Check if this is text.
    fn is_text(&self) -> bool;
    /// Check if this is a dictionary.
    fn is_dictionary(&self) -> bool;
    /// Check if this is a table.
    fn is_table(&self) -> bool;
    /// Check if this is a composition.
    fn is_composition(&self) -> bool;
    /// Check if this is a pattern.
    fn is_pattern(&self) -> bool;
}

/// Text.
pub trait Text<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Pt>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Pt>,
    Cm: Composition<Vl, Tx, Dc, Tb, Cm, Pt>,
    Pt: Pattern<Vl, Tx, Dc, Tb, Cm, Pt>,
> {
    fn as_str(&self) -> &str;
}

/// A dictionary.
pub trait Dictionary<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Pt>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Pt>,
    Cm: Composition<Vl, Tx, Dc, Tb, Cm, Pt>,
    Pt: Pattern<Vl, Tx, Dc, Tb, Cm, Pt>,
> {
    type EntryIterator<'b>: Iterator<Item=Entry<'b, Vl>> where Self: 'b, Vl: 'b;
    /// Number of entries in this dictionary.
    fn len(&self) -> usize;
    /// Check if this dictionary is empty.
    fn is_empty(&self) -> bool;
    /// Get the entry at an index.
    fn get_at(&self, index: usize) -> Option<Entry<Vl>>;
    /// Iterate over the entries in this dictionary.
    fn iter(&self) -> Self::EntryIterator<'_>;
}

/// A dictionary entry.
pub struct Entry<'a, St>(&'a str, &'a St);

/// A table.
pub trait Table<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Pt>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Pt>,
    Cm: Composition<Vl, Tx, Dc, Tb, Cm, Pt>,
    Pt: Pattern<Vl, Tx, Dc, Tb, Cm, Pt>,
> {
    /// Iterator over the rows in a table.
    type RowIterator<'b>: Iterator<Item=Box<[&'b Vl]>> where Self: 'b, Vl: 'b;
    /// Iterator over the entries in a table.
    type EntryIterator<'b>: Iterator<Item=&'b Vl> where Self: 'b, Vl: 'b;
    /// Number of entries in this table.
    fn len(&self) -> usize;
    /// Number of columns in this table.
    fn columns(&self) -> usize;
    /// Number of rows in this table.
    fn rows(&self) -> usize;
    /// Check if this table is empty.
    fn is_empty(&self) -> bool;
    /// Check if this table is a list.
    ///
    /// A list is a table with a single column.
    fn is_list(&self) -> bool;
    /// Check if this table is a tuple.
    ///
    /// A tuple is a table with a single row.
    fn is_tuple(&self) -> bool;
    /// Get the entry at
    fn get_entry_at(&self, row: usize, column: usize) -> Option<&Vl>;
    ///
    fn get_entry_at_index(&self, index: usize) -> Option<&Vl>;
    /// Get the row at an index.
    fn get_row_(&self, row: usize) -> Option<&[Vl]>;
    /// Iterate over the entries in this table.
    fn iter_entries(&self) -> Self::EntryIterator<'_>;
    /// Iterate over the rows in this table.
    fn iter_rows(&self) -> Self::RowIterator<'_>;
}

/// A pattern.
pub trait Pattern<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Pt>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Pt>,
    Cm: Composition<Vl, Tx, Dc, Tb, Cm, Pt>,
    Pt: Pattern<Vl, Tx, Dc, Tb, Cm, Pt>,
> {
    /// Iterator over pattern arguments.
    type ArgumentIterator<'b>: Iterator<Item=&'b Vl> + 'b where Self: 'b, Vl: 'b;
    /// Iterator over pattern attributes.
    type AttributeIterator<'b>: Iterator<Item=Attribute<'b>> + 'b where Self: 'b;
    /// Name of the pattern.
    fn name(&self) -> &str;
    /// Number of arguments.
    fn len(&self) -> usize;
    /// Check if this pattern has attributes.
    fn has_attributes(&self) -> bool;
    /// Check if this pattern has arguments.
    fn has_arguments(&self) -> bool;
    /// Get the argument at an index.
    fn get_argument_at(&self, index: usize) -> Option<&Vl>;
    /// Get the attribute by name.
    fn get_attribute_by_key(&self, key: &str) -> Option<AttributeValue<'_>>;
    /// Get the attribute by index.
    fn get_attribute_at(&self, index: usize) -> Option<Attribute<'_>>;
    /// Iterate over the arguments of this pattern.
    fn iter_arguments(&self) -> Self::ArgumentIterator<'_>;
    /// Iterate over the attributes of this pattern.
    fn iter_attributes(&self) -> Self::AttributeIterator<'_>;
}

/// An attribute of a pattern.
pub struct Attribute<'a>(&'a str, Option<&'a str>);

/// An attribute value of a pattern.
pub struct AttributeValue<'a>(Option<&'a str>);

/// A composition.
///
/// Corresponds to a textual composition of multiple data structures.
pub trait Composition<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Pt>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Pt>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Pt>,
    Cm: Composition<Vl, Tx, Dc, Tb, Cm, Pt>,
    Pt: Pattern<Vl, Tx, Dc, Tb, Cm, Pt>,
> {
    /// Iterator over the elements in a composition.
    type ElementIterator<'a>: Iterator<Item=Element<&'a Vl>> where Self: 'a, Vl: 'a;
    /// Number of elements in this composition.
    fn len(&self) -> usize;
    /// Get the element at an index.
    fn get_at(&self, index: usize) -> Option<Element<&Vl>>;
    /// Iterate over the elements in this composition.
    fn iter(&self) -> Self::ElementIterator<'_>;
}

/// An element in a composition.
pub enum Element<T> {
    Substance(T),
    Whitespace,
}

pub fn translate_escape_character(char: char) -> Result<char, ()> {
    match char {
        '{' => Ok('{'),
        '}' => Ok('}'),
        '[' => Ok('['),
        ']' => Ok(']'),
        '<' => Ok('<'),
        '>' => Ok('>'),
        '"' => Ok('"'),
        ':' => Ok(':'),
        ';' => Ok(';'),
        '|' => Ok('|'),
        '~' => Ok('~'),
        '#' => Ok('#'),
        '`' => Ok('`'),
        'n' => Ok('\n'),
        _ => Err(()),
    }
}
