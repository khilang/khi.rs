use std::fs::File;
use std::io::Read;
use khi::parse::{parse_dictionary_str, parse_expression_str, parse_table_str};

#[test]
fn test_examples() {
    parse_dictionary_str(&read_document_file("examples/aluminium.a")).unwrap();
    parse_table_str(&read_document_file("examples/elements.khi")).unwrap();
    parse_expression_str(&read_document_file("examples/equations.tex.khi")).unwrap();
    parse_expression_str(&read_document_file("examples/frontpage.html.khi")).unwrap();
    parse_expression_str(&read_document_file("examples/fruits.xml.khi")).unwrap();
    parse_dictionary_str(&read_document_file("examples/materials.khi")).unwrap();
    parse_table_str(&read_document_file("examples/multiline-quotes.khi")).unwrap();
    parse_table_str(&read_document_file("examples/primes.khi")).unwrap();
    parse_table_str(&read_document_file("examples/server-log.khi")).unwrap();
    parse_expression_str(&read_document_file("examples/style.khi")).unwrap();
}

fn read_document_file(path: &str) -> String {
    let mut file = File::open(path).unwrap();
    let mut document = String::new();
    file.read_to_string(&mut document).unwrap();
    document
}
