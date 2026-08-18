mod common;

use std::ops::Deref;
use indoc::indoc;
use khi::{Catenation, Dictionary, List, TaggedTuple, Text, Value};
use khi::parse::parse::{parse_dictionary_str, parse_list_str, parse_value_str};
use crate::common::{assert_dictionary, assert_invalid_table, assert_table, assert_tag, assert_terms, assert_text, assert_tuple, join_strings_in_value, strcmp};

#[test]
fn test_lexer() { // TODO

}

#[test]
fn test_terms() {
    let source = "A text term";
    let document = parse_value_str(source).unwrap();
    assert!(document.is_text());
    let source = "{k1: v1; k2: v2}";
    let document = parse_value_str(source).unwrap();
    assert!(document.is_dictionary());
    let source = "[e1; e2; e3]";
    let document = parse_value_str(source).unwrap();
    assert!(document.is_list());
    let source = "<Tag>:arg:arg";
    let document = parse_value_str(source).unwrap();
    assert!(document.is_tagged_tuple());
}

#[test]
fn test_expressions() {

}

#[test]
fn test_grouping() {
    let source = "{A text term}";
    let document = parse_value_str(source).unwrap();
    assert!(document.is_text());
}

#[test]
fn test_text_terms() {
    assert_text("Hello world!", "Hello world!");
    assert_text(" Hello world! ", "Hello world!");
    assert_text("Hello\tworld!", "Hello world!");
    assert_text("Hello\nworld!", "Hello world!");
    assert_text("R e d", "R e d");
    //assert_text("R ~ e ~ d", "Red");
    //assert_text("A<#>A  A<#>. A\\A  A\\.", "AA  A. AA  A.");
}

// #[test]
// fn test_composition() {
//     let source = "<p1>:arg1:arg2:<p3>:arg4:arg5:<p6>:<p7>:arg8";
//     let document = parse_value_str(source).unwrap();
//     assert!(document.is_tagged());
//     let p1 = document.as_tagged().unwrap();
//     assert_eq!(p1.get().unfold().len(), 3);
//     let p1arg = p1.get().unfold();
//     let p1arg = p1arg.get(2).unwrap();
//     assert!(p1arg.is_tagged());
//     let p2 = p1arg.as_tagged().unwrap();
//     assert_eq!(p2.get().unfold().len(), 3);
//     let p2arg = p2.get().unfold();
//     let p2arg = p2arg.get(2).unwrap();
//     assert!(p2arg.is_tagged());
//     let p3 = p2arg.as_tagged().unwrap().get();
//     assert!(p3.is_tagged());
// }

#[test]
fn test_expression() {
    assert_terms("", "Tp"); // Empty document returns a unit tuple.
    assert_terms("Text", "Tx");
    assert_terms("{k: v}", "Dc");
    assert_terms("[1|0;0|1]", "Ls");
    assert_terms("<P>", "Tp");
    assert_terms("{~} {Text [Table]}", "NlTxCt");
    //assert_terms("{Text [Table]}", "TxLs");
    assert_terms("{~}", "");
    //assert_terms("Text {Text} [Table] {k: v} <Dir>", "TxTxTxLsTxDcTxTp");
    //assert_terms("Text \\Text\\ [Table] {k: v} <Dir>", "TxTxTxTxTxLsTxDcTxTp");
}

#[test]
fn test_tuple() {
    assert_tuple("a a | b b", 2);
    assert_tuple("a | [b] | {c}", 3);
    assert_tuple("| a", 1);
    assert_tuple("| a | {b} | <c> | d", 4);
    assert_tuple("<a>:b:c | d d | <e>:f", 3);
    assert_tuple("<>: a", 1);
    assert_tuple("<>: a | b", 2);
    assert_tuple("| d d\n| <e>:f", 2);
    assert_tag("<a>: b", 1);
    assert_tag("<a>: b b | c", 2);
}

fn test_tagged_value() {
    assert_tuple("<>: <>: <>: <>", 1);
    assert_tuple("<>: <>: <>: { <> }", 1);
    assert_tuple("<>: <>: <>: { a | b }", 1);
}

#[test]
fn test_table() {
    // Test empty table
    let document = parse_value_str("[]").unwrap();
    assert!(document.is_list());
    let table = document.as_list().unwrap();
    assert_eq!(table.len(), 0);
    assert!(table.is_empty());
    // Test valid sequential notation.
    assert_table("", 0, 0);
    assert_table("1", 1, 1);
    assert_table("~", 1, 1);
    assert_table("1;", 1, 1);
    assert_table("~;", 1, 1);
    assert_table("1|0", 1, 2);
    assert_table("~|~", 1, 2);
    assert_table("1|0;", 1, 2);
    assert_table("1;0", 2, 1);
    assert_table("~;~", 2, 1);
    assert_table("1;0;", 2, 1);
    assert_table("1|0;0|1", 2, 2);
    assert_table("1|0|0;0|1|0;0|0|1", 3, 3);
    assert_table("1|~|~;~|1|~;~|~|1", 3, 3);
    assert_table("~|~|~;~|~|~;~|~|~", 3, 3);
    // Test invalid sequential notation.
    assert_invalid_table("1|0; ;");
    assert_invalid_table(";1|0;");
    assert_invalid_table("1|0|");
    assert_invalid_table("1|~|");
    assert_invalid_table("1| |0");

    // Test valid bullet notation.
    assert_table("> A > B > C", 3, 1);
    assert_table("> A | B | C", 1, 3);
    assert_table("> A | B > C | D", 2, 2);
    assert_table("> | A > B > C", 3, 1);
    assert_table("> A > B > | C", 3, 1);
    // Test invalid bullet notation.
    assert_invalid_table("> > A > B > C");
    assert_invalid_table("> > A > B > > C");
}

#[test]
fn test_tabulated_list() {
    assert_table("|1|", 1, 1);
    assert_table("|~|", 1, 1);
    assert_table("|1|1|", 1, 2);
    assert_table("|~|~|", 1, 2);
    assert_table("|1|0| |0|1|", 2, 2);
    assert_table("|~|~| |~|~|", 2, 2);
    assert_table("|1|~| |~|1|", 2, 2);
    assert_table("|1| |1|", 2, 1);
    assert_table("|1|0|0| |0|1|0| |0|0|1|", 3, 3);
    assert_invalid_table("|");
    assert_invalid_table("| |");
    assert_invalid_table("|a|b|c| |d|e|f");
    assert_invalid_table("|a|b|c| | |d|e|f");
}




#[test]
fn test_tilde_operator() {
    assert_terms("~", "");
    //assert_terms("~ ~", "");
    //assert_terms("A ~", "Tx");
    //assert_terms("A ~ B", "TxTx");
    //assert_terms("A ~ B ~ C", "TxTxTx");
    //assert_terms("{A} ~ {B}", "TxTx");
    //assert_terms("{A} ~ {B} ~ {C}", "TxTxTx");
    //assert_terms("{A}~{B}~{C}", "TxTxTx");
    //assert_terms("~{A} {B}~ ~{C}~", "Tx TxTx");
}

#[test]
fn test_dictionary() {
    // Test empty
    let source = "{}";
    let document = parse_value_str(source).unwrap();
    assert!(document.is_dictionary());
    let dictionary = document.as_dictionary().unwrap();
    assert_eq!(dictionary.len(), 0);
    assert!(dictionary.is_empty());
    // Test regular
    let source = "{k1: v1; k2: v2; k3: v3}";
    let document = parse_value_str(source).unwrap();
    assert!(document.is_dictionary());
    let dictionary = document.as_dictionary().unwrap();
    assert_eq!(dictionary.len(), 3);
    // Test trailing
    let source = "{k1: v1; k2: v2; k3: v3;}";
    let document = parse_value_str(source).unwrap();
    assert!(document.is_dictionary());
    let dictionary = document.as_dictionary().unwrap();
    assert_eq!(dictionary.len(), 3);
    assert_dictionary("", 0);
    assert_dictionary("k: v", 1);
    assert_dictionary("k: v;", 1);
    assert_dictionary("k: ~", 1);
    assert_dictionary("k1: v1; k2: v2", 2);
    assert_dictionary("k1: v1; k2: v2;", 2);
    assert_dictionary("k1: ~; k2: v2", 2);
    assert_dictionary("k1: ~; k2: ~", 2);
    assert_dictionary("k1: ~; k2: ~;", 2);
}


/// Test grammar rules where whitespace is required.
///
/// ```text
/// <aligned-dictionary> → <entry>_<aligned-dictionary>
/// <absolute-dictionary> → <inner-dictionary>_<absolute-dictionary'>
/// <absolute-dictionary'> → <section>_<absolute-dictionary>
/// <section> → <square-header>":"_<list>
/// <section> → <curly-header>":"_<inner-dictionary>
/// <section> → <curly-header>":"_<value>
/// <aligned-list> → ">"_<value>
/// <aligned-list> → ">"_<value>_<aligned-list>
/// <tabular-list> → "|" <inner-value> "|"_<tabular-list>
/// <tagged-list> → <tagged-value>_<tagged-list>
/// <tag> → "<"<word>_<attributes> ">"
/// <attributes> → <attribute>_<attributes>
/// ```
fn test_required_whitespace() {
    let source = "key: {value}key: value";
}

fn assert_err() {

}

#[test]
fn test_section() {
    let source = r#"
        a: b
        {c}:
        d: e
        [f]:
        g; h; i
    "#;
    let dictionary = parse_dictionary_str(source).unwrap();
    assert_eq!(dictionary.len(), 3);
    let c = dictionary.get("c").unwrap();
    assert!(c.is_dictionary());
    let c = c.as_dictionary().unwrap();
    assert_eq!(c.len(), 1);
}

#[test]
fn test_compound() {
    let src = "Aaaaa bbb cccc <d>:e:f";
    let parse = parse_value_str(src).unwrap();
    let compound = parse.as_catenation().unwrap();
    //assert!(compound.whitespace.get(0).unwrap());
}

//fn strcmp(strs: &[Result<ParsedValue, Vec<ParseError>>]) {
//    let t1 = strs[0].clone().unwrap();
//    let t1 = t1.as_text().unwrap().as_str();
//    let mut i = 1;
//    while i < strs.len() {
//        let ti = strs[i].clone().unwrap();
//        let ti = ti.as_text().unwrap().as_str();
//        assert_eq!(t1, ti);
//        i += 1;
//    }
//}

#[test]
fn test_strings_2() {
    let src1 = indoc! {r"
      key1: value1
      key2: value2
      key3: value3
      {S1}:
      key1: value1
      {S1 > Q1}:
      key1: value1
    "};
    let v = parse_dictionary_str(src1).unwrap();
}

#[test]
fn test_test() {
    let src = indoc! {r"
        <#>
          fasdfsd adfs afdasfsa
          dsfas fsdafas

          sdaf asdfasd
          asdfasd sdaf as
        <#>
    "};
    let v = parse_value_str(src).unwrap();
    let s = v.as_text().unwrap().as_str();
    eprintln!("[{s}]");
}

#[test]
fn test_test2() {
    let src = indoc! {r"
        a; b; c
    "};
    let v = parse_list_str(src);
    assert!(v.is_ok());
}

#[test]
fn test_test3() {
    let src = indoc! {r"
        a dfsg sdfgk fsdg psdfg

    "};
    let v = parse_value_str(src);
    assert!(v.is_ok());
}
