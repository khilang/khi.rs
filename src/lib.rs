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

//#[cfg(feature = "serde")]
//pub mod ser;
//#[cfg(feature = "serde")]
//pub mod de;

//mod fmt;
//mod model;

/// A value.
///
/// Corresponds to something that can be an element of a tuple, such as a real data
/// structure.
///
/// Is one of:
/// - nil
/// - text
/// - dictionary
/// - tuple
/// - table
/// - compound
/// - tag
pub trait Value<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tg: Tag<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
> {
    /// Get as text.
    fn as_text(&self) -> Option<&Tx>;
    /// Get as a dictionary.
    fn as_dictionary(&self) -> Option<&Dc>;
    /// Get as a tuple.
    fn as_tuple(&self) -> Option<&Tp>;
    /// Get as a table.
    fn as_table(&self) -> Option<&Tb>;
    /// Get as a compound.
    fn as_compound(&self) -> Option<&Cm>;
    /// Get as a tag.
    fn as_tag(&self) -> Option<&Tg>;
    /// Check if this is nil.
    fn is_nil(&self) -> bool;
    /// Check if this is text.
    fn is_text(&self) -> bool;
    /// Check if this is a dictionary.
    fn is_dictionary(&self) -> bool;
    /// Check if this is a tuple.
    fn is_tuple(&self) -> bool;
    /// Check if this is a table.
    fn is_table(&self) -> bool;
    /// Check if this is a compound.
    fn is_compound(&self) -> bool;
    /// Check if this is a tag.
    fn is_tag(&self) -> bool;
    /// Interprets this value as a tuple and gets the components.
    fn unfold(&self) -> Box<[&Vl]>;
}

/// Text.
pub trait Text<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tg: Tag<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
> {
    fn as_str(&self) -> &str;
}

/// A dictionary.
pub trait Dictionary<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tg: Tag<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
> {
    type EntryIterator<'b>: Iterator<Item=Entry<'b, Vl>> where Self: 'b, Vl: 'b;
    /// Number of entries in this dictionary.
    fn len(&self) -> usize;
    /// Check if this dictionary is empty.
    fn is_empty(&self) -> bool;
    /// Get the entry at an index.
    fn get(&self, index: usize) -> Option<Entry<Vl>>;
    /// Iterate over the entries in this dictionary.
    fn iter(&self) -> Self::EntryIterator<'_>;
}

/// A dictionary entry.
pub struct Entry<'a, St>(&'a str, &'a St);

/// A tuple.
pub trait Tuple<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tg: Tag<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
> {
    type TupleIterator<'b>: Iterator<Item=&'b Vl> where Self: 'b, Vl: 'b;
    /// Number of elements in the tuple.
    fn len(&self) -> usize;
    /// Check if this tuple is empty.
    fn is_empty(&self) -> bool;
    /// Get the element at an index.
    fn get(&self, index: usize) -> Option<&Vl>;
    /// Iterate over the elements in this tuple.
    fn iter(&self) -> Self::TupleIterator<'_>;
}

/// A table.
pub trait Table<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tg: Tag<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
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
    ///
    /// The entries of the list can be iterated over with [self.iter_entries].
    fn is_list(&self) -> bool;
    /// Get the entry at indices.
    fn get_entry(&self, row: usize, column: usize) -> Option<&Vl>;
    /// Get the row at an index.
    fn get_row(&self, row: usize) -> Option<&[Vl]>;
    /// Iterate over the entries in this table.
    fn iter_entries(&self) -> Self::EntryIterator<'_>;
    /// Iterate over the rows in this table.
    fn iter_rows(&self) -> Self::RowIterator<'_>;
}

/// A tag.
pub trait Tag<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tg: Tag<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
> {
    /// Iterator over tag attributes.
    type AttributeIterator<'b>: Iterator<Item=Attribute<'b>> + 'b where Self: 'b;
    /// Name of the tag.
    fn name(&self) -> &str;
    /// Check if this tag has attributes.
    fn has_attributes(&self) -> bool;
    /// Get the attribute by key.
    fn get_attribute_by(&self, key: &str) -> Option<AttributeValue<'_>>;
    /// Get the attribute by index.
    fn get_attribute_at(&self, index: usize) -> Option<Attribute<'_>>;
    /// Iterate over the attributes of this tag.
    fn iter_attributes(&self) -> Self::AttributeIterator<'_>;
    /// Get the tagged value.
    fn get(&self) -> &Vl;
}

/// An attribute of a tag.
pub struct Attribute<'a>(&'a str, Option<&'a str>);

/// An attribute value of a tag.
pub struct AttributeValue<'a>(Option<&'a str>);

/// A compound.
///
/// Corresponds to a textual composition of multiple data structures.
pub trait Compound<
    Vl: Value<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tb: Table<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
    Tg: Tag<Vl, Tx, Dc, Tb, Cm, Tp, Tg>,
> {
    /// Iterator over the elements in a compound.
    type ElementIterator<'a>: Iterator<Item=Element<&'a Vl>> where Self: 'a, Vl: 'a;
    /// Number of elements in this compound.
    fn len(&self) -> usize;
    /// Get the element at an index.
    fn get(&self, index: usize) -> Option<Element<&Vl>>;
    /// Iterate over the elements in this compound.
    fn iter(&self) -> Self::ElementIterator<'_>;
}

/// An element in a compound.
pub enum Element<T> {
    Solid(T),
    Space,
}

pub fn translate_escape_character(char: char) -> Result<char, ()> {
    match char {
        ':' => Ok(':'),
        ';' => Ok(';'),
        '|' => Ok('|'),
        '&' => Ok('&'),
        '~' => Ok('~'),
        '`' => Ok('`'),
        '\\' => Ok('\\'),
        '{' => Ok('{'),
        '}' => Ok('}'),
        '[' => Ok('['),
        ']' => Ok(']'),
        '<' => Ok('<'),
        '>' => Ok('>'),
        '#' => Ok('#'),
        'n' => Ok('\n'),
        't' => Ok('\t'),
        _ => Err(()),
    }
}
