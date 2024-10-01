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
pub mod pdm;
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
/// - compound
/// - tag
pub trait Value<
    Vl: Value<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Ls: List<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tg: Tagged<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
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
    /// Check if this is a compound.
    fn is_compound(&self) -> bool;
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
    /// Get as a compound.
    fn as_compound(&self) -> Option<&Cm>;
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
    /// Get as a compound.
    fn as_mut_compound(&mut self) -> Option<&mut Cm>;
    /// Iterate as a tuple.
    ///
    /// If the value is a tuple, its components are iterated over. Otherwise,
    /// the single value itself is iterated over.
    fn iter_as_tuple<'b>(&'b self) -> impl Iterator<Item=&'b Vl> where Vl: 'b;
}

/// Text.
pub trait Text<
    Vl: Value<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Ls: List<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tg: Tagged<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
> {
    fn as_str(&self) -> &str;
}

/// A tagged value.
pub trait Tagged<
    Vl: Value<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Ls: List<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tg: Tagged<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
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

/// A tuple.
pub trait Tuple<
    Vl: Value<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Ls: List<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tg: Tagged<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
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

/// A dictionary.
pub trait Dictionary<
    Vl: Value<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Ls: List<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tg: Tagged<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
> {
    type EntryIterator<'b>: Iterator<Item=(&'b str, &'b Vl)> where Self: 'b, Vl: 'b;
    /// Number of entries in this dictionary.
    fn len(&self) -> usize;
    /// Check if this dictionary is empty.
    fn is_empty(&self) -> bool;
    /// Get the entry at an index.
    fn get(&self, key: &str) -> Option<&Vl>;
    /// Get the entry at an index.
    fn get_mut(&mut self, key: &str) -> Option<&mut Vl>;
    /// Iterate over the entries in this dictionary.
    fn iter(&self) -> Self::EntryIterator<'_>;
}

/// A list.
pub trait List<
    Vl: Value<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Ls: List<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tg: Tagged<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
> {
    /// Iterator over the entries in a table.
    type ListIterator<'b>: Iterator<Item=&'b Vl> where Self: 'b, Vl: 'b;
    /// Number of entries in this list.
    fn len(&self) -> usize;
    /// Check if this list is empty.
    fn is_empty(&self) -> bool;
    /// Get the entry at index.
    fn get_element(&self, index: usize) -> Option<&Vl>;
    /// Iterate over the entries in this list.
    fn iter(&self) -> Self::ListIterator<'_>;
}

/// A compound.
pub trait Compound<
    Vl: Value<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tx: Text<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Dc: Dictionary<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Ls: List<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Cm: Compound<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tp: Tuple<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
    Tg: Tagged<Vl, Tx, Dc, Ls, Cm, Tp, Tg>,
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
    Element(T),
    Whitespace,
}

/// Get the character corresponding to an escaped character sequence.
pub fn translate_escape_character(char: char) -> Result<char, ()> {
    match char {
        ':' => Ok(':'),
        ';' => Ok(';'),
        '|' => Ok('|'),
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
