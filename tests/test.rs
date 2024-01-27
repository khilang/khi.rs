use std::ops::Deref;
use khi::parse::{parse_dictionary_str, parse_expression_str, parse_table_str, ParsedComposition, ParsedValue};
use khi::{Composition, Dictionary, Pattern, Value, Table, Element};

#[test]
pub fn test_lexer() {
    // let mut a = CharIter::new(source.chars());
    // let lex = unsafe{a.lex().unwrap_unchecked()};
    // for b in lex {
    //     let str = match b {
    //         Token::BracketOpening(_) => "BracketOpening",
    //         Token::BracketClosing(_) => "BracketClosing",
    //         Token::Semicolon(_) => "Semicolon",
    //         Token::Colon(_) => "Colon",
    //         Token::Comment(_) => "Comment",
    //         Token::Diamond(_) => "Diamond",
    //         Token::SequenceOpening(_) => "SequenceOpening",
    //         Token::SequenceClosing(_) => "SequenceClosing",
    //         Token::Word(_, _) => "Word",
    //         Token::Quote(_, _) => "Quote",
    //         Token::DirectiveOpening(_) => "DirectiveOpening",
    //         Token::ClosingTagOpening(_) => "ClosingTagOpening",
    //         Token::OpeningTagOpening(_) => "OpeningTagOpening",
    //         Token::DirectiveClosing(_) => "DirectiveClosing",
    //         Token::Whitespace(_) => "Whitespace",
    //         Token::End(_) => "End",
    //     };
    //     print!("{}\n", str);
    // }
}

#[test]
pub fn test_components() {
    let source = "A text argument";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_text());
    let source = "{k1: v1; k2: v2}";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_dictionary());
    let source = "[e1; e2; e3]";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_table());
    let source = "<dir>:arg:arg";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_pattern());
}

#[test]
pub fn test_expressions() {

}

#[test]
pub fn test_grouping() {
    let source = "{A text argument}";
    let document = parse_expression_str(source).unwrap();
    assert!(document.is_text());
}

#[test]
pub fn test_composition() {
    let source = "<dir1>:arg1:arg2:<>:<dir2>:arg3:arg4:<>:<dir3>:arg5";
    let expr = parse_expression_str(source).unwrap();
    assert!(expr.is_pattern());
    let dir1 = expr.as_pattern().unwrap();
    assert_eq!(dir1.len(), 3);
    let dir1arg = dir1.arguments.get(2).unwrap();
    assert!(dir1arg.is_pattern());
    let dir2 = dir1arg.as_pattern().unwrap();
    assert_eq!(dir2.len(), 3);
    let dir2arg = dir2.arguments.get(2).unwrap();
    assert!(dir2arg.is_pattern());
    let dir3 = dir2arg.as_pattern().unwrap();
    assert_eq!(dir3.len(), 1);
}

#[test]
pub fn test_dictionary_1() {
    let source = "{}";
    let expr = parse_expression_str(source).unwrap();
    assert!(expr.is_dictionary());
    let dict = expr.as_dictionary().unwrap();
    assert_eq!(dict.len(), 0);
}

#[test]
pub fn test_dictionary_2() {
    let source = "{k1: v1; k2: v2; k3: v3}";
    let expr = parse_expression_str(source).unwrap();
    assert!(expr.is_dictionary());
    let dict = expr.as_dictionary().unwrap();
    assert_eq!(dict.len(), 3);
}

#[test]
pub fn test_dictionary_3() {
    let source = "{k1: v1; k2: v2; k3: v3;}";
    let expr = parse_expression_str(source).unwrap();
    assert!(expr.is_dictionary());
    let dict = expr.as_dictionary().unwrap();
    assert_eq!(dict.len(), 3);
}

#[test]
fn test_expression() {
    assert_structure_form("Text", 1, "Tx");
    assert_structure_form("[1|0;0|1]", 1, "Tb");
    assert_structure_form("{k: v}", 1, "Dc");
    assert_structure_form("<Dir>", 1, "Dr");
    assert_structure_form("{~} {Text [Table]}", 2, "~ Cm");
    assert_structure_form("{~}", 0, "");
    assert_structure_form("", 0, "");
    assert_structure_form("Text {Text} [Table] {k: v} <Dir>", 5, "Tx Tx Tb Dc Dr");
}

#[test]
fn test_contraction() {
    assert_structure_form("R ~ G ~ B", 1, "Tx");
}

#[test]
fn test_constructor_notation() {
    assert_constructor("a a : b b", 2);
    assert_constructor("a : [b] : {c}", 3);
    assert_constructor("<a>:b:c : d d : <e>:f", 3);
    assert_directive("<a> : b", 1);
    assert_directive("<a> : b b : c", 2);
}

fn assert_constructor(source: &str, len: usize) {
    let expression = parse_expression_str(source).unwrap();
    let table = expression.as_table().unwrap();
    assert_eq!(len, table.columns());
}

fn assert_directive(source: &str, len: usize) {
    let expression = parse_expression_str(source).unwrap();
    let directive = expression.as_pattern().unwrap();
    assert_eq!(len, directive.len());
}

pub fn assert_structure_form(source: &str, length: usize, summary: &str) {
    let expr = parse_expression_str(source).unwrap();
    let summary2 = print_expression_form(&expr);
    assert!(summary2.eq(summary));
}

fn print_expression_form(structure: &ParsedValue) -> String {
    let mut summary = String::new();
    match structure {
        ParsedValue::Text(_, _, _) => summary = format!("{}Tx", summary),
        ParsedValue::Table(_, _, _) => summary = format!("{}Tb", summary),
        ParsedValue::Dictionary(_, _, _) => summary = format!("{}Dc", summary),
        ParsedValue::Pattern(_, _, _) => summary = format!("{}Dr", summary),
        ParsedValue::Composition(c, _, _) => {
            for e in c.iter() {
                match e {
                    Element::Substance(c) => {
                        match c {
                            ParsedValue::Text(_, _, _) => summary = format!("{}Tx", summary),
                            ParsedValue::Table(_, _, _) => summary = format!("{}Tb", summary),
                            ParsedValue::Dictionary(_, _, _) => summary = format!("{}Dc", summary),
                            ParsedValue::Pattern(_, _, _) => summary = format!("{}Dr", summary),
                            ParsedValue::Composition(_, _, _) => summary = format!("{}Cm", summary),
                            ParsedValue::Nil(_, _) => summary = format!("{}~", summary),
                        }
                    }
                    Element::Whitespace => summary = format!("{} ", summary),
                }
            }
        }
        ParsedValue::Nil(_, _) => summary = format!("{}", summary),
    }
    summary
}

#[test]
fn test_escape_sequences() {
    assert_text("`{", "{");
    assert_text("`}", "}");
    assert_text("`[", "[");
    assert_text("`]", "]");
    assert_text("`<", "<");
    assert_text("`>", ">");
    assert_text("`\"", "\"");
    assert_text("`:", ":");
    assert_text("`;", ";");
    assert_text("`|", "|");
    assert_text("`~", "~");
    assert_text("`#", "#");
    assert_text("``", "`");
    assert_text("`n", "\n");
    // Invalid escapes
    assert_invalid_expression("`a");
    assert_invalid_expression("`1");
}

fn assert_text(source: &str, str: &str) {
    let expr = parse_expression_str(source).unwrap();
    assert!(expr.is_text());
    let text = expr.as_text().unwrap().str.deref();
    assert!(text.eq(str));
}

#[test]
fn test_repeated_escape_sequences() {
    assert_structure_form("::", 1, "Tx");
    assert_structure_form(":::", 1, "Tx");
    assert_structure_form(";;", 1, "Tx");
    assert_structure_form(";;;", 1, "Tx");
    assert_structure_form("||", 1, "Tx");
    assert_structure_form("|||", 1, "Tx");
    assert_structure_form("~~", 1, "Tx");
    assert_structure_form("~~~", 1, "Tx");
    assert_structure_form("<<", 1, "Tx");
    assert_structure_form("<<<", 1, "Tx");
    assert_structure_form(">>", 1, "Tx");
    assert_structure_form(">>>", 1, "Tx");
}

#[test]
fn test_hash() {
    // Text
    assert_structure_form("#a", 1, "Tx");
    assert_structure_form("#1", 1, "Tx");
    assert_structure_form("#?", 1, "Tx");
    assert_structure_form("#`:", 1, "Tx");
    assert_structure_form("A#B", 1, "Tx");
    // Comments
    assert_structure_form("##", 0, "");
    assert_structure_form("# ", 0, "");
    assert_structure_form("#", 0, "");
    // Invalid sequence
    assert_invalid_expression("#{");
    assert_invalid_expression("#}");
    assert_invalid_expression("#[");
    assert_invalid_expression("#]");
    assert_invalid_expression("#<");
    assert_invalid_expression("#>");
    assert_invalid_expression("#\"");
    assert_invalid_expression("#:");
    assert_invalid_expression("#;");
    assert_invalid_expression("#|");
    assert_invalid_expression("#~");
}

fn assert_invalid_expression(source: &str) {
    assert!(parse_expression_str(source).is_err());
}

#[test]
fn test_tables() {
    // Test empty table
    let expression = parse_expression_str("[]").unwrap();
    assert!(expression.is_table());
    let table = expression.as_table().unwrap();
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
    // Test valid tabular notation.
    assert_table("|1|", 1, 1);
    assert_table("|~|", 1, 1);
    assert_table("|1|1|", 1, 2);
    assert_table("|~|~|", 1, 2);
    assert_table("|1|0| |0|1|", 2, 2);
    assert_table("|~|~| |~|~|", 2, 2);
    assert_table("|1|~| |~|1|", 2, 2);
    assert_table("|1| |1|", 2, 1);
    assert_table("|1|0|0| |0|1|0| |0|0|1|", 3, 3);
    // Test invalid tabular notation.
    assert_invalid_table("|");
    assert_invalid_table("| |");
    assert_invalid_table("|a");
    assert_invalid_table("|a|b|c| |d|e|f");
    assert_invalid_table("|a|b|c| | |d|e|f");
    assert_invalid_table("|a|b|c| |d|e|");
    assert_invalid_table("|a|b| |d|e|f|");
    assert_invalid_table("|a|b| |~|e|f|");
    // Test valid bullet notation.
    assert_table("> A > B > C", 3, 1);
    assert_table("> A | B | C", 1, 3);
    assert_table("> A | B > C | D", 2, 2);
    // Test invalid bullet notation.
    assert_invalid_table("> > A > B > C");
    assert_invalid_table("> > A > B > > C");
    assert_invalid_table("> | A > B > C");
    assert_invalid_table("> A > B > | C");
}

fn assert_table(source: &str, rows: usize, columns: usize) {
    let table = parse_table_str(source).unwrap();
    assert_eq!(table.rows(), rows);
    assert_eq!(table.columns(), columns);
    assert_eq!(table.len(), rows * columns);
}

fn assert_invalid_table(source: &str) {
    let table = parse_table_str(source);
    assert!(table.is_err());
}

#[test]
fn test_inline_quote() {
    assert_string("\"a b c\"", "a b c");
    assert_string("\" a b  c d  e f  g h \"", " a b  c d  e f  g h ");
    assert!(parse_expression_str("\"a b\nc d\"").is_err());
}

// #[test] TODO: FIX
fn test_multiline_quote() {
    let eq = "def main():\n  print(\"Hello world\")\nmain()\n";
    // Test indentation.
    let src = "<#>\n  def main():\n    print(\"Hello world\")\n  main()\n<#>";
    assert_string(src, eq);
    // Equal increase in indentation.
    let src = "<#>\n    def main():\n      print(\"Hello world\")\n    main()\n<#>";
    assert_string(src, eq);
    // Start immediately after <#>.
    let src = "<#>def main():\n     print(\"Hello world\")\n   main()\n<#>";
    assert_string(src, eq);
    // Remove newline before <#>.
    let src = "<#>def main():\n     print(\"Hello world\")\n   main()<#>";
    assert_string(src, eq);
}

fn assert_string(src: &str, eq: &str) {
    let expression = parse_expression_str(src).unwrap();
    let str = expression.as_text().unwrap().str.deref();
    assert_eq!(str, eq);
}

#[test]
fn test_component_separator() {
    assert_structure_form("~", 0, "");
    assert_structure_form("~ ~", 0, "");
    assert_structure_form("A ~", 1, "Tx");
    assert_structure_form("A ~ B", 2, "Tx");
    assert_structure_form("A ~ B ~ C", 3, "Tx");
    assert_structure_form("{A} ~ {B}", 2, "TxTx");
    assert_structure_form("{A} ~ {B} ~ {C}", 3, "TxTxTx");
    assert_structure_form("{A}~{B}~{C}", 3, "TxTxTx");
    assert_structure_form("~{A}~{B}~ ~{C}~", 3, "TxTxTx");
}

#[test]
fn test_dictionaries() {
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
    let dictionary = parse_dictionary_str(source).unwrap();
    assert_eq!(dictionary.len(), size);
}
