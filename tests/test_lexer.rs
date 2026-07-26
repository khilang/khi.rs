mod common;

use crate::common::{assert_invalid_expression, assert_terms, assert_text};

#[test]
fn test_escape_sequences() {
    assert_text(r"\``\", "`");
    assert_text(r"\`\\", "\\");
    assert_text(r"\`n\", "\n");
    assert_text(r"\`t\", "\t");
    // Invalid escapes
    assert_invalid_expression(r"\`a\");
    assert_invalid_expression(r"\`1\");
    assert_invalid_expression(r"\`:\");
}

#[test]
fn test_repeated_escape_sequences() {
    assert_terms("::", "Tx");
    assert_terms(";;", "Tx");
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
    //assert_terms("#`:", "Tx");
    assert_terms("#>>", "TxTx");
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
