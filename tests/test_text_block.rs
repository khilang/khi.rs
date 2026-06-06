use indoc::indoc;
use khi::parse::parse::parse_value_str;
use khi::{Text, Value};
use crate::common::assert_text;

mod common;

// parse_assert("<# rx>\n  def main():\n    print(\"Hello world\")\n  <#>", "\n  def main():\n    print(\"Hello world\")\n  ");

#[test]
fn test_header_flag() {
    parse_assert("<#:rh>\n  def main():\n    print(\"Hello world\")\n  <#>", "  def main():\n    print(\"Hello world\")\n  ");
}

#[test]
fn test_footer_flag() {
    parse_assert("<#:rf>\n  def main():\n    print(\"Hello world\")\n  <#>", "\n  def main():\n    print(\"Hello world\")\n");
}

#[test]
fn test_leading_flag() {
    parse_assert("<#:rl>\n  def main():\n    print(\"Hello world\")\n  <#>", "\ndef main():\nprint(\"Hello world\")\n");
}

#[test]
fn test_trailing_flag() {
    parse_assert("<#:rt>\n  def main():\n    print(\"Hello world\")\n  <#>", "\n  def main():\n    print(\"Hello world\")\n");
}

#[test]
fn test_excess_flag() {
    parse_assert("<#:rx>\n  def main():\n    print(\"Hello world\")\n  <#>", "\ndef main():\n  print(\"Hello world\")\n");
}

#[test]
fn test_newline_flag() {
    parse_assert("<#:rn>\n  def main():\n    print(\"Hello world\")\n  <#>", "  def main():    print(\"Hello world\")  ");
}

fn parse_assert(source: &str, string: &str) {
    let val = parse_value_str(source).unwrap();
    let text = val.as_text().unwrap().as_str();
    assert_eq!(text, string);
}

#[test]
fn test_default_text_block() {
    let expect = "def main():\n  print(\"Hello world\")\nmain()\n";
    // Test indentation.
    let source = "<#>\n  def main():\n    print(\"Hello world\")\n  main()\n<#>";
    assert_text(source, expect);
    // Equal increase in indentation.
    let source = "<#>\n    def main():\n      print(\"Hello world\")\n    main()\n<#>";
    assert_text(source, expect);
    // Start immediately after <#>.
    let source = "<#>  def main():\n    print(\"Hello world\")\n  main()\n<#>";
    assert_text(source, expect);
    assert_text(indoc!(r#"
        <#>
          def main():
            print("Hello world")

          def add(a, b):
            return a + b

          main()
        <#>
    "#), indoc!(r#"
      def main():
        print("Hello world")

      def add(a, b):
        return a + b

      main()
    "#));
}

#[test]
fn test_text_block_configuration() {
    let source = "  \n  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n  ";
    assert_text(
        &format!("{}{}{}", "<#>", source, "<#>"),
        "level 1\nlevel 1\n  level 2\n  level 2\nlevel 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<#:r>", source, "<#>"),
        "  \n  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n  ",
    );
    assert_text(
        &format!("{}{}{}", "<#:rf>", source, "<#>"),
        "  \n  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<#:rh>", source, "<#>"),
        "  level 1  \n  level 1\n    level 2  \n    level 2\n  level 1\n  ",
    );
    assert_text(
        &format!("{}{}{}", "<#:rx>", source, "<#>"),
        "\nlevel 1  \nlevel 1\n  level 2  \n  level 2\nlevel 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<#:rt>", source, "<#>"),
        "\n  level 1\n  level 1\n    level 2\n    level 2\n  level 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<#:rl>", source, "<#>"),
        "\nlevel 1  \nlevel 1\nlevel 2  \nlevel 2\nlevel 1\n",
    );
    assert_text(
        &format!("{}{}{}", "<#:rn>", source, "<#>"),
        "    level 1    level 1    level 2      level 2  level 1  ",
    );
}






