use indoc::indoc;
use khi::parse::parse::parse_value_str;
use crate::common::{assert_err, join_strings_in_value, strcmp};

mod common;

#[test]
fn test_strings_1() {
    let src1 = r"\Paragraph number one.\Paragraph number two.\";
    let src2 = indoc! {r"
        \Paragraph number one.
        \Paragraph number two.
    "};
    //let src3 = indoc! {r"
    //    Paragraph number \
    //    one.\
    //    `n\
    //    Paragraph number \
    //    two.\
    //"};
    //let src4 = indoc! {r"
    //    Paragraph number \
    //    one.`n\
    //    Paragraph number \
    //    two.\
    //"};
    //let src5 = indoc! {r"
    //    Paragraph number
    //    one.\
    //    \`n
    //    Paragraph number
    //    two.\
    //"};

    strcmp(&[
        join_strings_in_value(&parse_value_str(src1).unwrap()),
        join_strings_in_value(&parse_value_str(src2).unwrap()),
        //join_strings_in_value(&parse_value_str(src3).unwrap()),
        //join_strings_in_value(&parse_value_str(src4).unwrap()),
        //join_strings_in_value(&parse_value_str(src5).unwrap()),
    ]);
}

#[test]
fn test_strings_2() {
    let src1 = r"\Paragraph number one.Paragraph number two. three.\";
    let src2 = indoc! {r"
        \Paragraph number one.
        \Paragraph number two.
         three.
    "};
    //let src3 = indoc! {r"
    //    Paragraph number \
    //    one.\
    //    `n\
    //    Paragraph number \
    //    two.\
    //"};
    //let src4 = indoc! {r"
    //    Paragraph number \
    //    one.`n\
    //    Paragraph number \
    //    two.\
    //"};
    //let src5 = indoc! {r"
    //    Paragraph number
    //    one.\
    //    \`n
    //    Paragraph number
    //    two.\
    //"};

    strcmp(&[
        join_strings_in_value(&parse_value_str(src1).unwrap()),
        join_strings_in_value(&parse_value_str(src2).unwrap()),
        //join_strings_in_value(&parse_value_str(src3).unwrap()),
        //join_strings_in_value(&parse_value_str(src4).unwrap()),
        //join_strings_in_value(&parse_value_str(src5).unwrap()),
    ]);
}

/// Cannot close an open transcription.
#[test]
fn test_strings_3() {
    let src2 = indoc! {r"
        \Paragraph number one.
        \Paragraph number two.\
    "};
    assert_err(parse_value_str(src2));
}
