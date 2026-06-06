use crate::common::assert_text;

mod common;

#[test]
fn test_transcription() {
    assert_text("\\a b c\\", "a b c");
    assert_text("\\ a b  c d  e f  g h \\", " a b  c d  e f  g h ");
    //assert_text("\\a b\nc d\\ e f", "a bc d e f");
}
