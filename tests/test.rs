use udl::lex::{CharIter, Token};
use udl::parse::parse_expression_document;


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
pub fn test_arguments() {
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
    assert!(document.is_sequence());
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

}


#[test]
pub fn test_composition() {
    let source = "<dir1>:arg1:arg2:<>:<dir2>:arg3:arg4:<>:<dir3>:arg5";
    let expr = parse_expression_document(source).unwrap();
    assert!(expr.is_directive());
    let mut dir1 = expr.into_directive().unwrap();
    assert_eq!(dir1.length(), 3);
    let dir1arg = dir1.arguments.remove(2);
    assert!(dir1arg.is_directive());
    let mut dir2 = dir1arg.into_directive().unwrap();
    assert_eq!(dir2.length(), 3);
    let dir2arg = dir2.arguments.remove(2);
    assert!(dir2arg.is_directive());
    let dir3 = dir2arg.into_directive().unwrap();
    assert_eq!(dir3.length(), 1);
}


#[test]
pub fn test_tags() {
    let source = "<+$><+Sum>:k:1:n 3k^2 - 2k <-><->";
    let dir1 = parse_expression_document(source).unwrap();
    assert_eq!(dir1.length(), 1);
    assert!(dir1.is_directive());
    let mut dir1 = dir1.into_directive().unwrap();
    assert_eq!(dir1.length(), 1);
    let expr2 = dir1.arguments.remove(0);
    assert_eq!(expr2.length(), 1);
    assert!(expr2.is_directive());
    let dir2 = expr2.into_directive().unwrap();
    assert_eq!(dir2.length(), 4);
}
