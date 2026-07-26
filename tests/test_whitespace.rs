use khi::parse::parse::parse_value_str;
use crate::common::{assert_err, assert_ok};

mod common;

/// ```
/// <compact-elements> → ":"<compact-element>
///                    | ":"<compact-element><compact-elements>
/// ```
#[test]
fn test_compact_elements() {
    assert_ok(parse_value_str(r"<tag>:e1:e2:e3"));
    assert_err(parse_value_str(r"<tag>: e1:e2:e3"));
    assert_err(parse_value_str(r"<tag>:e1: e2:e3"));
    assert_err(parse_value_str(r"<tag>:e1:e2 :e3"));
}

/// ```
/// <tag> → "<"<key>">"
///       | "<"<key>_<attributes> ">"
///       | "<"">"
/// ```
#[test]
fn test_tag() {
    assert_ok(parse_value_str("<tag>"));
    assert_err(parse_value_str("< tag>"));
    assert_err(parse_value_str("<tag >"));

    assert_ok(parse_value_str("<tag a1=v1 a2>"));
    assert_ok(parse_value_str("<tag a1=v1 a2 >"));
    assert_err(parse_value_str("< tag a1=v1 a2>"));

    assert_ok(parse_value_str("<>"));
    assert_err(parse_value_str("< >"));
}

/// ```
/// <attributes> → <attribute>
///              | <attribute>_<attributes>
/// ```
#[test]
fn test_attributes() {
    assert_ok(parse_value_str(r"<tag a1=\v1\ a2>"));
    assert_err(parse_value_str(r"<tag a1=\v1\a2>"));
}

/// ```
/// <attribute> → <key>
///             | <key>"="<text-unit>
/// ```
#[test]
fn test_attribute() {
    assert_ok(parse_value_str(r"<tag a1=v1>"));
    assert_err(parse_value_str(r"<tag a1 =v1>"));
    assert_err(parse_value_str(r"<tag a1= v1>"));
}



/// ```
/// <delimited-dictionary> → <entry>
///                        | <entry> ";"
///                        | <entry> ";" <delimited-dictionary>
/// ```
#[test]
fn test_delimited_dictionary() {
    assert_ok(parse_value_str(r"{A: a; B: b}"));
    assert_ok(parse_value_str(r"{A: a ;B: b}"));
}


/// ```
/// <aligned-dictionary> → <entry>
///                      | <entry>_<aligned-dictionary>
/// ```
#[test]
fn test_aligned_dictionary() {
    assert_ok(parse_value_str(r"{A: a B: b}"));
    assert_err(parse_value_str(r"{A: \a\B: b}"));
    assert_ok(parse_value_str("{A: \\a\nB: b}"));
}

/// ```
/// <sectioned-dictionary>  → <sectioned-dictionary'>
///                         | <regular-dictionary>_<sectioned-dictionary'>
/// <sectioned-dictionary'> → <section>
///                         | <section>_<sectioned-dictionary'>
/// ```
#[test]
fn test_sectioned_dictionary() {
    assert_ok(parse_value_str(r"{ k1: v1 {S1}: k2: v2 }"));
    assert_err(parse_value_str(r"{ k1: v1{S1}: k2: v2 }"));

    assert_ok(parse_value_str(r"{ k1: v1 {S1}: k2: v2 {S2}: k3: v3 }"));
    assert_err(parse_value_str(r"{ k1: v1 {S1}: k2: v2{S2}: k3: v3 }"));
}


/// ```
/// <section> → <curly-header>_<value>
///           | <curly-header>
///           | <curly-header>_<regular-dictionary>
///           | <square-header>
///           | <square-header>_<list>
/// ```
#[test]
fn test_section() {
    assert_ok(parse_value_str(r"{ {S1}: v1 }"));
    assert_err(parse_value_str(r"{ {S1}:v1 }"));

    assert_ok(parse_value_str(r"{ {S1}: k1: v1 }"));
    assert_err(parse_value_str(r"{ {S1}:k1: v1 }"));

    assert_ok(parse_value_str(r"{ [S1]: v1; v2 }"));
    assert_err(parse_value_str(r"{ [S1]:v1; v2 }"));
}

/// ```
/// <curly-header> → "{"<header-key>"}"":"
/// ```
#[test]
fn test_curly_header() {
    assert_ok(parse_value_str(r"{ {S1}: v1 }"));
    assert_err(parse_value_str(r"{ { S1}: v1 }"));
    assert_err(parse_value_str(r"{ {S1 }: v1 }"));
    assert_err(parse_value_str(r"{ {S1} : v1 }"));
}

/// ```
/// <square-header> → "["<header-key>"]"":"
/// ```
#[test]
fn test_square_header() {
    assert_ok(parse_value_str(r"{ [S1]: v1 }"));
    assert_err(parse_value_str(r"{ [ S1]: v1 }"));
    assert_err(parse_value_str(r"{ [S1 ]: v1 }"));
    assert_err(parse_value_str(r"{ [S1] : v1 }"));
}

/// ```
/// <header-key> → <key>
///              | <key>"[""]"
///              | <key>_">"_<header-key>
/// ```
#[test]
fn test_header_key() {
    assert_ok(parse_value_str(r"{ {k1 > k2[]}: v1 }"));
    assert_err(parse_value_str(r"{ {k1 > k2 []}: v1 }"));
    assert_err(parse_value_str(r"{ {k1 > k2[ ]}: v1 }"));

    assert_err(parse_value_str(r"{ {k1> k2[]}: v1 }"));
    assert_err(parse_value_str(r"{ {k1 >k2[]}: v1 }"));
}

/// ```
/// <entry> → <key>":"_<value>
/// ```
#[test]
fn test_entry() {
    assert_ok(parse_value_str(r"{ k1: v1 }"));
    assert_err(parse_value_str(r"{ k1 : v1 }"));
    assert_err(parse_value_str(r"{ k1:v1 }"));
}

/// ```
/// <bulleted-list> → ">"_<value>
///                 | ">"_<value>_<bulleted-list>
/// ```
#[test]
fn test_bulleted_list() {
    assert_ok(parse_value_str(r"[ > v1 > v2 ]"));
    assert_err(parse_value_str(r"[ >v1 > v2 ]"));

    assert_err(parse_value_str(r"[ > v1> v2 ]"));
}

/// ```
/// <tagged-list> → <tagged-tuple>
///               | <tagged-tuple>_<tagged-list>
/// ```
#[test]
fn test_tagged_list() {
    assert_ok(parse_value_str(r"[ <T1>: v1 <T2>: v2 ]"));
    assert_err(parse_value_str(r"[ <T1>: v1<T2>: v2 ]"));
}

/// ```
/// <tabulated-list> → <open-elements'> "|"
///                  | <open-elements'> "|"_<tabulated-list>
/// ```
#[test]
fn test_tabulated_list() {
    assert_ok(parse_value_str(r"[ |v1|v2| |v3|v4| ]"));
    assert_err(parse_value_str(r"[ |v1|v2||v3|v4| ]"));
}
