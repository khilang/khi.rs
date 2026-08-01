use std::ops::Deref;
use khi::parse::parse::{parse_dictionary_str, parse_list_str, parse_value_str};
use khi::parse::pdm::ParsedValue;
use khi::{Catenation, Dictionary, Element, List, TaggedTuple, Text, Value};

pub fn assert_tuple(source: &str, len: usize) {
    let expression = parse_value_str(source).unwrap();
    if expression.is_tagged_tuple() {
        let tuple = expression.as_tagged_tuple().unwrap();
        assert_eq!(tuple.len(), len);
    }
}

pub fn assert_tag(source: &str, len: usize) {
    let document = parse_value_str(source).unwrap();
    assert!(document.is_tagged_tuple());
    let tag = document.as_tagged_tuple().unwrap();
    assert_eq!(tag.len(), len);
}

pub fn assert_terms(source: &str, expect: &str) {
    let document = parse_value_str(source).unwrap();
    let summary = summarize_terms(&document);
    assert_eq!(summary, expect);
}

pub fn assert_dictionary(source: &str, size: usize) {
    let document = parse_dictionary_str(source).unwrap();
    assert_eq!(document.len(), size);
}

pub fn join_strings_in_value(value: &ParsedValue) -> String {
    match value {
        ParsedValue::Text(s, _, _) => {
            s.as_str().to_string()
        }
        ParsedValue::Catenation(c, _, _) => {
            let mut string = String::new();
            for e in c.iter() {
                match e {
                    Element::Element(e) => {
                        match e {
                            ParsedValue::Text(t, _, _) => {
                                string.push_str(t.as_str())
                            }
                            _ => unimplemented!(),
                        }
                    }
                    Element::Separator => {
                        string.push(' ');
                    }
                }
            }
            return string;
        }
        _ => unimplemented!()
    }
}

pub fn assert_ok<T, E>(value: Result<T, E>) {
    assert!(value.is_ok());
}

pub fn assert_err<T, E>(value: Result<T, E>) {
    assert!(value.is_err());
}

pub fn strcmp(strs: &[String]) {
    let t1 = &strs[0];
    let mut i = 1;
    while i < strs.len() {
        let ti = &strs[i];
        assert_eq!(t1, ti);
        i += 1;
    }
}

pub fn assert_text(source: &str, expect: &str) {
    let document = parse_value_str(source).unwrap();
    assert!(document.is_text());
    let string = document.as_text().unwrap().str.deref();
    assert_eq!(string, expect);
}

pub fn assert_table(source: &str, rows: usize, columns: usize) {
    let document = parse_list_str(source).unwrap();
    assert_eq!(document.len(), rows);
    //assert_eq!(document.columns(), columns);
    //assert_eq!(document.len(), rows * columns);
}

pub fn assert_invalid_table(source: &str) {
    let document = parse_list_str(source);
    assert!(document.is_err());
}

pub fn assert_invalid_expression(source: &str) {
    assert!(parse_value_str(source).is_err());
}

pub fn summarize_terms(value: &ParsedValue) -> String {
    let mut summary = String::new();
    match value {
        ParsedValue::Nil(..) => summary = format!("{}", summary),
        ParsedValue::Text(..) => summary = format!("{}Tx", summary),
        ParsedValue::Dictionary(..) => summary = format!("{}Dc", summary),
        ParsedValue::List(..) => summary = format!("{}Ls", summary),
        ParsedValue::Catenation(compound, ..) => {
            for element in compound.iter() {
                match element {
                    Element::Element(c) => {
                        match c {
                            ParsedValue::Nil(..) => summary = format!("{}Nl", summary),
                            ParsedValue::Text(..) => summary = format!("{}Tx", summary),
                            ParsedValue::Dictionary(..) => summary = format!("{}Dc", summary),
                            ParsedValue::List(..) => summary = format!("{}Ls", summary),
                            ParsedValue::Catenation(..) => summary = format!("{}Ct", summary),
                            ParsedValue::TaggedTuple(..) => summary = format!("{}Tp", summary),
                        }
                    }
                    Element::Separator => summary = format!("{} ", summary),
                }
            }
        }
        ParsedValue::TaggedTuple(..) => summary = format!("{}Tp", summary),
    }
    summary
}

