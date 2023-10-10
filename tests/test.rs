use khi::parse::{parse_dictionary_document, parse_expression_document, parse_table_document, ParsedComponent, ParsedExpression};
use khi::{Component, Expression, Table, Text, WhitespaceOption};

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
    let document = parse_expression_document(source).unwrap();
    assert_eq!(document.length(), 1);
    assert!(document.is_text());
    let source = "{ k1: v1; k2: v2 }";
    let document = parse_expression_document(source).unwrap();
    assert_eq!(document.length(), 1);
    assert!(document.is_dictionary());
    let source = "[ e1; e2; e3 ]";
    let document = parse_expression_document(source).unwrap();
    assert_eq!(document.length(), 1);
    assert!(document.is_table());
    let source = "<dir>:arg:arg";
    let document = parse_expression_document(source).unwrap();
    assert_eq!(document.length(), 1);
    assert!(document.is_directive());
}

#[test]
pub fn test_expressions() {

}

#[test]
pub fn test_grouping() {
    let source = "{ A text argument }";
    let expr = parse_expression_document(source).unwrap();
    assert_eq!(expr.length(), 1);
    assert!(expr.is_text());
}

#[test]
pub fn test_composition() {
    let source = "<dir1>:arg1:arg2:<>:<dir2>:arg3:arg4:<>:<dir3>:arg5";
    let expr = parse_expression_document(source).unwrap();
    assert!(expr.is_directive());
    let dir1 = expr.conform_directive().unwrap();
    assert_eq!(dir1.length(), 3);
    let dir1arg = dir1.arguments.get(2).unwrap();
    assert!(dir1arg.is_directive());
    let dir2 = dir1arg.as_directive().unwrap();
    assert_eq!(dir2.length(), 3);
    let dir2arg = dir2.arguments.get(2).unwrap();
    assert!(dir2arg.is_directive());
    let dir3 = dir2arg.as_directive().unwrap();
    assert_eq!(dir3.length(), 1);
}

#[test]
pub fn test_dictionary_1() {
    let source = "{#}";
    let expr = parse_expression_document(source).unwrap();
    assert_eq!(expr.length(), 1);
    assert!(expr.is_dictionary());
    let dict = expr.conform_dictionary().unwrap();
    assert_eq!(dict.size(), 0);
}

#[test]
pub fn test_dictionary_2() {
    let source = "{ k1: v1; k2: v2; k3: v3 }";
    let expr = parse_expression_document(source).unwrap();
    assert_eq!(expr.length(), 1);
    assert!(expr.is_dictionary());
    let dict = expr.conform_dictionary().unwrap();
    assert_eq!(dict.size(), 3);
}

#[test]
pub fn test_dictionary_3() {
    let source = "{ k1: v1; k2: v2; k3: v3; #}";
    let expr = parse_expression_document(source).unwrap();
    assert_eq!(expr.length(), 1);
    assert!(expr.is_dictionary());
    let dict = expr.conform_dictionary().unwrap();
    assert_eq!(dict.size(), 3);
}

#[test]
fn test_expression() {
    assert_expression("Text", 1, "Tx");
    assert_expression("[1|0;0|1]", 1, "Tb");
    assert_expression("{k: v}", 1, "Dc");
    assert_expression("<Dir>", 1, "Dr");
    assert_expression("{} {Text [Table]}", 2, "Em Cm");
    assert_expression("{}", 0, "");
    assert_expression("", 0, "");
    assert_expression("Text {Text} [Table] {k: v} <Dir>", 5, "Tx Tx Tb Dc Dr");
}

pub fn assert_expression(source: &str, length: usize, summary: &str) {
    let expr = parse_expression_document(source).unwrap();
    assert_eq!(expr.length(), length);
    let summary2 = textify_expression(&expr);
    assert!(summary2.eq(summary));
}

fn textify_expression(expression: &ParsedExpression) -> String {
    let mut summary = String::new();
    for a in expression.iter_components_with_whitespace() {
        summary = match a {
            WhitespaceOption::Component(ParsedComponent::Empty(..)) => format!("{}Em", summary),
            WhitespaceOption::Component(ParsedComponent::Text(..)) => format!("{}Tx", summary),
            WhitespaceOption::Component(ParsedComponent::Table(..)) => format!("{}Tb", summary),
            WhitespaceOption::Component(ParsedComponent::Dictionary(..)) => format!("{}Dc", summary),
            WhitespaceOption::Component(ParsedComponent::Directive(..)) => format!("{}Dr", summary),
            WhitespaceOption::Component(ParsedComponent::Compound(..)) => format!("{}Cm", summary),
            WhitespaceOption::Whitespace => format!("{} ", summary),
        };
    };
    summary
}

#[test]
fn test_tables() {
    // Test valid sequential notation.
    assert_table("", 0, 0);
    assert_table("1", 1, 1);
    assert_table(":", 1, 1);
    assert_table("1;", 1, 1);
    assert_table(":;", 1, 1);
    assert_table("1|0", 1, 2);
    assert_table(":|:", 1, 2);
    assert_table("1|0;", 1, 2);
    assert_table("1;0", 2, 1);
    assert_table(":;:", 2, 1);
    assert_table("1;0;", 2, 1);
    assert_table("1|0;0|1", 2, 2);
    assert_table("1|0|0;0|1|0;0|0|1", 3, 3);
    assert_table("1|:|:;:|1|:;:|:|1", 3, 3);
    assert_table(":|:|:;:|:|:;:|:|:", 3, 3);
    // Test invalid sequential notation.
    assert_invalid_table("1|0; ;");
    assert_invalid_table(";1|0;");
    assert_invalid_table("1|0|");
    assert_invalid_table("1|:|");
    assert_invalid_table("1| |0");
    // Test valid tabular notation.
    assert_table("|1|", 1, 1);
    assert_table("|:|", 1, 1);
    assert_table("|1|1|", 1, 2);
    assert_table("|:|:|", 1, 2);
    assert_table("|1|0| |0|1|", 2, 2);
    assert_table("|:|:| |:|:|", 2, 2);
    assert_table("|1|:| |:|1|", 2, 2);
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
    assert_invalid_table("|a|b| |:|e|f|");
}

fn assert_table(source: &str, rows: usize, columns: usize) {
    let table = parse_table_document(source).unwrap();
    assert_eq!(table.rows(), rows);
    assert_eq!(table.columns(), columns);
    assert_eq!(table.size(), rows * columns);
}

fn assert_invalid_table(source: &str) {
    let table = parse_table_document(source);
    assert!(table.is_err());
}

#[test]
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
    let expression = parse_expression_document(src).unwrap();
    let str = expression.get(0).unwrap().as_text().unwrap().as_str();
    assert_eq!(str, eq);
}

#[test]
fn test_dictionaries() {
    assert_dictionary("", 0);
    assert_dictionary("k:v", 1);
    assert_dictionary("k:v;", 1);
    assert_dictionary("k1:v1;k2:v2", 2);
    assert_dictionary("k1:v1;k2:v2;", 2);
    assert_dictionary("k1;", 1);
    assert_dictionary("k1;k2;", 2);
}

fn assert_dictionary(source: &str, size: usize) {
    let dictionary = parse_dictionary_document(source).unwrap();
    assert_eq!(dictionary.size(), size);
}
