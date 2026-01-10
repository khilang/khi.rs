extern crate core;

#[cfg(feature = "parse")]
pub mod parse;

//#[cfg(feature = "enc")]
//pub mod enc;

#[cfg(feature = "html")]
pub mod html;

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
/// - list
/// - catenation
/// - tag
pub trait Value<
    Va: Value<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tx: Text<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Dc: Dictionary<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ls: List<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ca: Catenation<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tp: Tuple<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tg: Tagged<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
> {
    /// Check if this is text.
    fn is_text(&self) -> bool;
    /// Check if this is a tag.
    fn is_tagged(&self) -> bool;
    /// Check if this is a tuple.
    fn is_tuple(&self) -> bool;
    /// Check if this is a dictionary.
    fn is_dictionary(&self) -> bool;
    /// Check if this is a table.
    fn is_list(&self) -> bool;
    /// Check if this is a catenation.
    fn is_catenation(&self) -> bool;
    /// Check if this is nil.
    fn is_nil(&self) -> bool;
    /// Get as text.
    fn as_text(&self) -> Option<&Tx>;
    /// Get as a tagged value.
    fn as_tagged(&self) -> Option<&Tg>;
    /// Get as a tuple.
    fn as_tuple(&self) -> Option<&Tp>;
    /// Get as a dictionary.
    fn as_dictionary(&self) -> Option<&Dc>;
    /// Get as a table.
    fn as_list(&self) -> Option<&Ls>;
    /// Get as a catenation.
    fn as_catenation(&self) -> Option<&Ca>;
    /// Get as text.
    fn as_mut_text(&mut self) -> Option<&mut Tx>;
    /// Get as a tagged value.
    fn as_mut_tagged(&mut self) -> Option<&mut Tg>;
    /// Get as a tuple.
    fn as_mut_tuple(&mut self) -> Option<&mut Tp>;
    /// Get as a dictionary.
    fn as_mut_dictionary(&mut self) -> Option<&mut Dc>;
    /// Get as a table.
    fn as_mut_list(&mut self) -> Option<&mut Ls>;
    /// Get as a catenation.
    fn as_mut_catenation(&mut self) -> Option<&mut Ca>;
    /// Iterate as a tuple.
    ///
    /// If the value is a tuple, its elements are iterated over. Otherwise,
    /// the single value itself is iterated over.
    fn iter_as_tuple<'b>(&'b self) -> impl Iterator<Item=ParsedTupleElement<'b, &'b Va>> where Va: 'b;
    /// Get the length of this value as a tuple.
    ///
    /// If the value is a tuple, its length is returned. Otherwise, 1 is
    /// returned.
    fn len_as_tuple(&self) -> usize;
}

/// Text.
pub trait Text<
    Va: Value<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tx: Text<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Dc: Dictionary<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ls: List<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ca: Catenation<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tp: Tuple<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tg: Tagged<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
> {
    fn as_str(&self) -> &str;
}

/// A tagged value.
pub trait Tagged<
    Va: Value<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tx: Text<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Dc: Dictionary<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ls: List<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ca: Catenation<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tp: Tuple<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tg: Tagged<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
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
    fn get(&self) -> &Tp;
}

/// An attribute of a tag.
pub struct Attribute<'a>(pub &'a str, pub Option<&'a str>);

/// An attribute value of a tag.
pub enum AttributeValue<'a> {
    Flag,
    Value(&'a str),
}

#[derive(Clone)]
pub enum ParsedTupleElement<'a, Va> {
    Element(Va),
    NamedElement(&'a str, Va),
}

/// A tuple.
pub trait Tuple<
    Va: Value<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tx: Text<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Dc: Dictionary<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ls: List<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ca: Catenation<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tp: Tuple<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tg: Tagged<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
> {
    type TupleIterator<'b>: Iterator<Item=ParsedTupleElement<'b, &'b Va>> where Self: 'b, Va: 'b;
    type TupleIteratorMut<'b>: Iterator<Item=ParsedTupleElement<'b, &'b mut Va>> where Self: 'b, Va: 'b;
    /// Number of elements in the tuple.
    fn len(&self) -> usize;
    /// Check if this tuple is empty.
    fn is_empty(&self) -> bool;
    /// Get the element at index.
    fn get(&self, index: usize) -> Option<ParsedTupleElement<&Va>>;
    /// Get the element at index as mutable.
    fn get_mut(&mut self, index: usize) -> Option<ParsedTupleElement<&mut Va>>;
    /// Iterate over the elements in this tuple.
    fn iter(&self) -> Self::TupleIterator<'_>;
    fn iter_mut(&mut self) -> Self::TupleIteratorMut<'_>;
}

/// A dictionary.
pub trait Dictionary<
    Va: Value<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tx: Text<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Dc: Dictionary<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ls: List<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ca: Catenation<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tp: Tuple<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tg: Tagged<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
> {
    type EntryIterator<'b>: Iterator<Item=(&'b str, &'b Va)> where Self: 'b, Va: 'b;
    type EntryIteratorMut<'b>: Iterator<Item=(&'b str, &'b mut Va)> where Self: 'b, Va: 'b;
    /// Number of entries in this dictionary.
    fn len(&self) -> usize;
    /// Check if this dictionary is empty.
    fn is_empty(&self) -> bool;
    /// Get the entry at index.
    fn get(&self, key: &str) -> Option<&Va>;
    /// Get the entry at index as mutable.
    fn get_mut(&mut self, key: &str) -> Option<&mut Va>;
    /// Iterate over the entries in this dictionary.
    fn iter(&self) -> Self::EntryIterator<'_>;
    fn iter_mut(&mut self) -> Self::EntryIteratorMut<'_>;
}

/// A list.
pub trait List<
    Va: Value<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tx: Text<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Dc: Dictionary<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ls: List<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ca: Catenation<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tp: Tuple<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tg: Tagged<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
> {
    /// Iterator over the entries in a table.
    type ListIterator<'b>: Iterator<Item=&'b Va> where Self: 'b, Va: 'b;
    type ListIteratorMut<'b>: Iterator<Item=&'b mut Va> where Self: 'b, Va: 'b;
    /// Number of entries in this list.
    fn len(&self) -> usize;
    /// Check if this list is empty.
    fn is_empty(&self) -> bool;
    /// Get the element at index.
    fn get(&self, index: usize) -> Option<&Va>;
    /// Get the element at index as mutable.
    fn get_mut(&mut self, index: usize) -> Option<&mut Va>;
    /// Iterate over the entries in this list.
    fn iter(&self) -> Self::ListIterator<'_>;
    fn iter_mut(&mut self) -> Self::ListIteratorMut<'_>;
}

/// A catenation.
pub trait Catenation<
    Va: Value<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tx: Text<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Dc: Dictionary<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ls: List<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Ca: Catenation<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tp: Tuple<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
    Tg: Tagged<Va, Tx, Dc, Ls, Ca, Tp, Tg>,
> {
    /// Iterator over the elements in a catenation.
    type ElementIterator<'a>: Iterator<Item=&'a Element<Va>> where Self: 'a, Va: 'a;
    type ElementIteratorMut<'a>: Iterator<Item=&'a mut Element<Va>> where Self: 'a, Va: 'a;
    /// Number of elements in this catenation.
    fn len(&self) -> usize;
    /// Get the element at an index.
    fn get(&self, index: usize) -> Option<&Element<Va>>;
    fn get_mut(&mut self, index: usize) -> Option<&mut Element<Va>>;
    /// Iterate over the elements in this catenation.
    fn iter(&self) -> Self::ElementIterator<'_>;
    fn iter_mut(&mut self) -> Self::ElementIteratorMut<'_>;
}

/// An element in a catenation.
#[derive(Clone)]
pub enum Element<T> {
    Element(T),
    Whitespace,
}

/// Get the character corresponding to an escaped character sequence.
pub fn translate_escape_character(char: char) -> Result<char, ()> {
    match char {
        '`' => Ok('`'),
        '\\' => Ok('\\'),
        'n' => Ok('\n'),
        't' => Ok('\t'),
        _ => Err(()),
    }
}
