//! Khi data structures.

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

/// An expression.
///
/// An expression encodes a data structure. It consists of components which provide
/// the information needed to reconstruct the data structure.
pub trait Expression<
    Ex: Expression<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tx: Text<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dc: Dictionary<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tb: Table<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dr: Directive<Ex, Tx, Dc, Tb, Dr, Cm>,
    Cm: Component<Ex, Tx, Dc, Tb, Dr, Cm>,
> {
    type ComponentIterator<'a>: Iterator<Item=&'a Cm> where Self: 'a, Cm: 'a;
    type ComponentIteratorWithWhitespace<'a>: Iterator<Item=WhitespaceOption<&'a Cm>> where Self: 'a, Cm: 'a;
    /// Number of components the structure consists of.
    fn length(&self) -> usize;
    /// Check if this is a structure with 0 components.
    fn is_empty(&self) -> bool;
    /// Check if this is a structure with 1 component.
    fn is_unary(&self) -> bool;
    /// Check if this is a structure with 2 or more components.
    fn is_compound(&self) -> bool;
    /// Get the component at index.
    fn get(&self, index: usize) -> Option<&Cm>;
    /// Iterate over the components and the whitespace in the expression.
    fn iter_components_with_whitespace(&self) -> Self::ComponentIteratorWithWhitespace<'_>;
    /// Iterate over the components in the expression.
    fn iter_components(&self) -> Self::ComponentIterator<'_>;
    /// Get as text.
    fn conform_text(&self) -> Option<&Tx>;
    /// Get as a table.
    fn conform_table(&self) -> Option<&Tb>;
    /// Get as a dictionary.
    fn conform_dictionary(&self) -> Option<&Dc>;
    /// Get as a directive.
    fn conform_directive(&self) -> Option<&Dr>;
    /// Get as an expression component.
    fn as_component(&self) -> &Cm;
    /// Check if this is a structure with a single text component.
    fn is_text(&self) -> bool;
    /// Check if this is a structure with a single table component.
    fn is_table(&self) -> bool;
    /// Check if this is a structure with a single dictionary component.
    fn is_dictionary(&self) -> bool;
    /// Check if this is a structure with a single directive component.
    fn is_directive(&self) -> bool;
}

pub enum WhitespaceOption<T> {
    Component(T),
    Whitespace,
}

/// Text.
pub trait Text<
    Ex: Expression<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tx: Text<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dc: Dictionary<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tb: Table<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dr: Directive<Ex, Tx, Dc, Tb, Dr, Cm>,
    Cm: Component<Ex, Tx, Dc, Tb, Dr, Cm>,
> {
    fn as_str(&self) -> &str;
}

/// A dictionary.
pub trait Dictionary<
    Ex: Expression<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tx: Text<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dc: Dictionary<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tb: Table<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dr: Directive<Ex, Tx, Dc, Tb, Dr, Cm>,
    Cm: Component<Ex, Tx, Dc, Tb, Dr, Cm>,
> {
    type EntryIterator<'b>: Iterator<Item=(&'b str, &'b Ex)> where Self: 'b, Ex: 'b;
    /// Number of entries in this dictionary.
    fn length(&self) -> usize;
    fn is_empty(&self) -> bool;
    /// Get value by key. Returns first match.
    fn get_by(&self, key: &str) -> Option<&Ex>;
    /// Get value at index.
    fn get_at(&self, index: usize) -> Option<(&str, &Ex)>;
    /// Iterate over the entries in this dictionary.
    fn iter_entries(&self) -> Self::EntryIterator<'_>;
}

/// A table.
pub trait Table<
    Ex: Expression<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tx: Text<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dc: Dictionary<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tb: Table<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dr: Directive<Ex, Tx, Dc, Tb, Dr, Cm>,
    Cm: Component<Ex, Tx, Dc, Tb, Dr, Cm>,
> {
    type RowIterator<'b>: Iterator<Item=Box<[&'b Ex]>> where Self: 'b, Ex: 'b;
    type ListIterator<'b>: Iterator<Item=&'b Ex> where Self: 'b, Ex: 'b;
    fn is_empty(&self) -> bool;

    /// Number of columns in this table.
    fn columns(&self) -> usize;
    /// Number of rows in this table.
    fn rows(&self) -> usize;
    /// Number of cells in this table.
    fn size(&self) -> usize;
    fn get(&self, row: usize, column: usize) -> Option<&Ex>;
    fn get_row(&self, row: usize) -> Option<&[Ex]>;
    fn iter_rows(&self) -> Self::RowIterator<'_>;
    /// Check if this table is a list.
    /// A list is a table with a single column.
    fn is_list(&self) -> bool;
    fn len_as_list(&self) -> usize;
    fn get_list_element(&self, index: usize) -> Option<&Ex>;
    fn iter_list_elements(&self) -> Self::ListIterator<'_>;
    /// Check if this table is a tuple.
    /// A tuple is a table with a single row.
    fn is_tuple(&self) -> bool;
}

/// A directive.
pub trait Directive<
    Ex: Expression<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tx: Text<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dc: Dictionary<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tb: Table<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dr: Directive<Ex, Tx, Dc, Tb, Dr, Cm>,
    Cm: Component<Ex, Tx, Dc, Tb, Dr, Cm>,
> {
    type ArgumentIterator<'b>: Iterator<Item=&'b Cm> + 'b where Self: 'b, Cm: 'b;
    type AttributeIterator<'b>: Iterator<Item=(&'b str, &'b Ex)> + 'b where Self: 'b, Tx: 'b, Ex: 'b;
    fn label(&self) -> &str;
    fn length(&self) -> usize;
    fn has_attributes(&self) -> bool;
    fn has_arguments(&self) -> bool;
    fn get_argument(&self, index: usize) -> Option<&Cm>;
    fn get_attribute(&self, key: &str) -> Option<&Ex>;
    fn get_attribute_at(&self, index: usize) -> Option<(&str, &Ex)>;
    fn iter_arguments(&self) -> Self::ArgumentIterator<'_>;
    fn iter_attributes(&self) -> Self::AttributeIterator<'_>;
}

/// A component.
///
/// A component is either:
/// - an expression component
/// - a text component
/// - a table component
/// - a dictionary component
/// - a directive component
pub trait Component<
    Ex: Expression<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tx: Text<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dc: Dictionary<Ex, Tx, Dc, Tb, Dr, Cm>,
    Tb: Table<Ex, Tx, Dc, Tb, Dr, Cm>,
    Dr: Directive<Ex, Tx, Dc, Tb, Dr, Cm>,
    Cm: Component<Ex, Tx, Dc, Tb, Dr, Cm>,
> {
    /// Get as a structure.
    fn as_expression(&self) -> &Ex;
    /// Get as text.
    fn as_text(&self) -> Option<&Tx>;
    /// Get as a table.
    fn as_table(&self) -> Option<&Tb>;
    /// Get as a dictionary.
    fn as_dictionary(&self) -> Option<&Dc>;
    /// Get as a directive.
    fn as_directive(&self) -> Option<&Dr>;
    /// Check if this is a text component or an expression with a single text element.
    fn is_text(&self) -> bool;
    /// Check if this is a table component or an expression with a single table element.
    fn is_table(&self) -> bool;
    /// Check if this is a dictionary component or an expression with a single dictionary element.
    fn is_dictionary(&self) -> bool;
    /// Check if this is a directive component or an expression with a single directive element.
    fn is_directive(&self) -> bool;
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
        '?' => Ok('?'),
        '`' => Ok('`'),
        'n' => Ok('\n'),
        _ => Err(()),
    }
}
