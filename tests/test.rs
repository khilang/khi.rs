use std::ops::Deref;
use indoc::indoc;
use khi::{Catenation, Dictionary, Tagged, Value, List, Element, Tuple, Text};
use khi::parse::parse::{parse_dictionary_str, parse_list_str, parse_value_str};
use khi::parse::parse::parser::ParseError;
use khi::parse::pdm::ParsedValue;

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
    assert!(document.is_tagged());
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
    assert_text("R ~ e ~ d", "Red");
    assert_text("A<#>A  A<#>. A\\A  A\\.", "AA  A. AA  A.");
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
    assert_terms("[1|0;0|1]", "Tb");
    assert_terms("<P>", "Pt");
    assert_terms("{~} {Text [Table]}", "Nl Cm");
    assert_terms("{Text [Table]}", "Tx Tb");
    assert_terms("{~}", "");
    assert_terms("Text {Text} [Table] {k: v} <Dir>", "Tx Tx Tb Dc Pt");
    assert_terms("Text \"Text\" [Table] {k: v} <Dir>", "Tx Tb Dc Pt");
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

fn assert_tuple(source: &str, len: usize) {
    let expression = parse_value_str(source).unwrap();
    if expression.is_tuple() {
        let tuple = expression.as_tuple().unwrap();
        assert_eq!(tuple.len(), len);
    }
}

fn assert_tag(source: &str, len: usize) {
    let document = parse_value_str(source).unwrap();
    assert!(document.is_tagged());
    let tag = document.as_tagged().unwrap();
    let tuple = tag.get();
    assert_eq!(tuple.len(), len);
}

fn assert_terms(source: &str, expect: &str) {
    let document = parse_value_str(source).unwrap();
    let summary = summarize_terms(&document);
    assert_eq!(summary, expect);
}

fn summarize_terms(value: &ParsedValue) -> String {
    let mut summary = String::new();
    match value {
        ParsedValue::Nil(..) => summary = format!("{}", summary),
        ParsedValue::Text(..) => summary = format!("{}Tx", summary),
        ParsedValue::Dictionary(..) => summary = format!("{}Dc", summary),
        ParsedValue::Tuple(..) => summary = format!("{}Tp", summary),
        ParsedValue::List(..) => summary = format!("{}Tb", summary),
        ParsedValue::Catenation(compound, ..) => {
            for element in compound.iter() {
                match element {
                    Element::Element(c) => {
                        match c {
                            ParsedValue::Nil(..) => summary = format!("{}Nl", summary),
                            ParsedValue::Text(..) => summary = format!("{}Tx", summary),
                            ParsedValue::Dictionary(..) => summary = format!("{}Dc", summary),
                            ParsedValue::Tuple(..) => summary = format!("{}Tp", summary),
                            ParsedValue::List(..) => summary = format!("{}Tb", summary),
                            ParsedValue::Catenation(..) => summary = format!("{}Cm", summary),
                            ParsedValue::Tagged(..) => summary = format!("{}Pt", summary),
                        }
                    }
                    Element::Whitespace => summary = format!("{} ", summary),
                }
            }
        }
        ParsedValue::Tagged(..) => summary = format!("{}Pt", summary),
    }
    summary
}

#[test]
fn test_escape_sequences() {
    assert_text("``", "`");
    assert_text("`\\", "\\");
    assert_text("`n", "\n");
    assert_text("`t", "\t");
    // Invalid escapes
    assert_invalid_expression("`a");
    assert_invalid_expression("`1");
    assert_invalid_expression("[`");
    assert_invalid_expression("n`");
}

#[test]
fn test_repeated_escape_sequences() {
    assert_terms("::", "Tx");
    assert_terms(";;", "Tx");
    assert_terms("||", "Tx");
    assert_terms("~~", "Tx");
    assert_terms("<<", "Tx");
    assert_terms(">>", "Tx");
}

#[test]
fn test_hash() {
    // Text
    assert_terms("#a", "Tx");
    assert_terms("#1", "Tx");
    assert_terms("#?", "Tx");
    assert_terms("#`:", "Tx");
    assert_terms("#>>", "Tx");
    assert_terms("A#B", "Tx");
    // Comments
    assert_terms("##", "Tp");
    assert_terms("# ", "Tp");
    assert_terms("#", "Tp");
    // Invalid sequence
    assert_invalid_expression("#:");
    assert_invalid_expression("#;");
    assert_invalid_expression("#|");
    assert_invalid_expression("#~");
    assert_invalid_expression("#\\");
    assert_invalid_expression("#{");
    assert_invalid_expression("#}");
    assert_invalid_expression("#[");
    assert_invalid_expression("#]");
    assert_invalid_expression("#<");
    assert_invalid_expression("#>");
}

fn assert_invalid_expression(source: &str) {
    assert!(parse_value_str(source).is_err());
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

fn assert_table(source: &str, rows: usize, columns: usize) {
    let document = parse_list_str(source).unwrap();
    assert_eq!(document.len(), rows);
    //assert_eq!(document.columns(), columns);
    //assert_eq!(document.len(), rows * columns);
}

fn assert_invalid_table(source: &str) {
    let document = parse_list_str(source);
    assert!(document.is_err());
}

#[test]
fn test_transcription() {
    assert_text("\\a b c\\", "a b c");
    assert_text("\\ a b  c d  e f  g h \\", " a b  c d  e f  g h ");
    assert_text("\\a b\nc d\\ e f", "a bc d e f");
}

#[test]
fn test_default_text_block() {
    let expect = "def main():\n  print(\"Hello world\")\nmain()\n";
    // Test indentation.
    let source = "<#>\n  def main():\n    print(\"Hello world\")\n  main()\n<#>";
    assert_text(source, expect);
    // Equal increase in indentation.
    let source = "<#>\n    def main():\n      print(\"Hello world\")\n    main()\n<#>";
    assert_text(source, expect);
    // Start immediately after <#>.
    let source = "<#>  def main():\n    print(\"Hello world\")\n  main()\n<#>";
    assert_text(source, expect);
}

#[test]
fn test_text_block_configuration() {
    let source = "  \n  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n  ";
    assert_text(
        &format!("{}{}{}", "<#>", source, "<#>"),
        "level 1  \nlevel 1\n  level 2  \n  level 2\nlevel 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<# r>", source, "<#>"),
        "  \n  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n  ",
    );
    assert_text(
        &format!("{}{}{}", "<# rf>", source, "<#>"),
        "  \n  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<# rh>", source, "<#>"),
        "  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n  ",
    );
    assert_text(
        &format!("{}{}{}", "<# rx>", source, "<#>"),
        "\nlevel 1  \nlevel 1\n  level 2  \n  level 2\nlevel 1\n  ",
    );
    assert_text(
        &format!("{}{}{}", "<# rt>", source, "<#>"),
        "\n  level 1\n  level 1\n    level 2\n    level 2\n  level 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<# rl>", source, "<#>"),
        "\nlevel 1  \nlevel 1\nlevel 2  \nlevel 2\nlevel 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<# rn>", source, "<#>"),
        "    level 1    level 1    level 2      level 2  level 1  ",
    );
}

fn assert_text(source: &str, expect: &str) {
    let document = parse_value_str(source).unwrap();
    assert!(document.is_text());
    let string = document.as_text().unwrap().str.deref();
    assert_eq!(string, expect);
}

#[test]
fn test_tilde_operator() {
    assert_terms("~", "");
    assert_terms("~ ~", "");
    assert_terms("A ~", "Tx");
    assert_terms("A ~ B", "Tx");
    assert_terms("A ~ B ~ C", "Tx");
    assert_terms("{A} ~ {B}", "TxTx");
    assert_terms("{A} ~ {B} ~ {C}", "TxTxTx");
    assert_terms("{A}~{B}~{C}", "TxTxTx");
    assert_terms("~{A} {B}~ ~{C}~", "Tx TxTx");
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
    assert_dictionary("k:v", 1);
    assert_dictionary("k:v;", 1);
    assert_dictionary("k:~", 1);
    assert_dictionary("k1:v1;k2:v2", 2);
    assert_dictionary("k1:v1;k2:v2;", 2);
    assert_dictionary("k1:~;k2:v2", 2);
    assert_dictionary("k1:~;k2:~", 2);
    assert_dictionary("k1:~;k2:~;", 2);
}

fn assert_dictionary(source: &str, size: usize) {
    let document = parse_dictionary_str(source).unwrap();
    assert_eq!(document.len(), size);
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

/// Test grammar rules where whitespace is disallowed.
///
/// ```text
/// <tagged-value> → <tag>":"_<value>
///
///
/// ```
fn test_disallowed_whitespace() {

}

/// Test grammar rules where whitespace is optional.
fn test_optional_whitespace() {

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

#[test]
fn test_strings_1() {
    let src1 = r"\Paragraph number one.`nParagraph number two.\";
    let src2 = indoc! {r"
        \Paragraph number
        \ one.
        \`n
        \Paragraph number
        \ two.
    "};
    let src3 = indoc! {r"
        Paragraph number \
        one.\
        `n\
        Paragraph number \
        two.\
    "};
    let src4 = indoc! {r"
        Paragraph number \
        one.`n\
        Paragraph number \
        two.\
    "};
    let src5 = indoc! {r"
        Paragraph number
        one.\
        \`n
        Paragraph number
        two.\
    "};

    strcmp(&[parse_value_str(src1), parse_value_str(src2), parse_value_str(src3), parse_value_str(src4), parse_value_str(src5)]);
}

fn strcmp(strs: &[Result<ParsedValue, Vec<ParseError>>]) {
    let t1 = strs[0].clone().unwrap();
    let t1 = t1.as_text().unwrap().as_str();
    let mut i = 1;
    while i < strs.len() {
        let ti = strs[i].clone().unwrap();
        let ti = ti.as_text().unwrap().as_str();
        assert_eq!(t1, ti);
        i += 1;
    }
}

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
