use std::fs::File;
use std::io::Read;
use khi::parse::{parse_dictionary_str, parse_value_str, parse_list_str};

#[test]
fn test_example_article_aluminium() {
    parse_dictionary_str(&read_document_file("examples/aluminium.a")).unwrap();
}

#[test]
fn test_example_elements_table() {
    parse_list_str(&read_document_file("examples/elements.khi")).unwrap();
}

#[test]
fn test_example_equations_tex() {
    parse_value_str(&read_document_file("examples/equations.tex.khi")).unwrap();
}

#[test]
fn test_example_frontpage_html() {
    parse_value_str(&read_document_file("examples/frontpage.html.khi")).unwrap();
}

#[test]
fn test_example_fruits_xml() {
    parse_value_str(&read_document_file("examples/fruits.xml.khi")).unwrap();
}

#[test]
fn test_example_inventory_log() {
    parse_list_str(&read_document_file("examples/inventory-log.khi")).unwrap();
}

#[test]
fn test_example_materials() {
    parse_dictionary_str(&read_document_file("examples/materials.khi")).unwrap();
}

#[test]
fn test_example_primes() {
    parse_list_str(&read_document_file("examples/primes.khi")).unwrap();
}

#[test]
fn test_example_server_log() {
    parse_list_str(&read_document_file("examples/server-log.khi")).unwrap();
}

#[test]
fn test_example_style() {
    parse_value_str(&read_document_file("examples/style.khi")).unwrap();
}

#[test]
fn test_example_text_blocks() {
    parse_list_str(&read_document_file("examples/text-blocks.khi")).unwrap();
}

#[test]
fn test_example_words() {
    parse_list_str(&read_document_file("examples/words.khi")).unwrap();
}

fn read_document_file(path: &str) -> String {
    let mut file = File::open(path).unwrap();
    let mut document = String::new();
    file.read_to_string(&mut document).unwrap();
    document
}
