use khi::parse::parse::{parse_dictionary_str, parse_value_str};
use crate::common::{assert_err, assert_ok};

mod common;



/// Sections can only be assigned once.
#[test]
fn test_assign_once() {
    assert_ok(parse_dictionary_str(r"{S1}: A: a; B: b {S2}: C: c"));
    assert_err(parse_dictionary_str(r"{S1}: A: a; B: b {S1}: C: c"));

    assert_ok(parse_dictionary_str(r"S1: s1 {S2}: A: a; B: b"));
    assert_err(parse_dictionary_str(r"S1: s1 {S1}: A: a; B: b"));
}

/// Supersections must be assigned before subsections.
#[test]
fn test_assign_order() {
    assert_ok(parse_dictionary_str(r"{S1}: A: a; B: b {S1 > S2}: C: c"));
    assert_err(parse_dictionary_str(r"{S1 > S2}: C: c {S1}: A: a; B: b"));
}

/// Cannot interact with section from inner dictionary.
#[test]
fn test_reassign_section() {
    assert_err(parse_dictionary_str(r"S2: s2 {S1}: A: a; B: b {S2}: C: c"));
    assert_err(parse_dictionary_str(r"{S1}: A: a; B: b; S2: s2 {S1 > S2}: C: c"));
}

/// Sections can only be assigned once.
#[test]
fn test_append() {
    assert_ok(parse_dictionary_str(r"{S1[]}: A: a; B: b {S1[]}: A: a; B: b"));
    assert_err(parse_dictionary_str(r"S1: [s1; s2] {S1[]}: A: a; B: b"));
}
