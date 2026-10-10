use crate::format;
use lspower::lsp::FormattingOptions;

mod comment;
mod common;
mod ctrl_stmt;
mod horizontal_single;
mod let_block;
mod math;
mod module;
mod printer;
mod space;
mod syntax;

fn test_tmpl(input: &str, expect: &str) {
    let option = FormattingOptions {
        tab_size: 4,
        insert_spaces: true,
        ..Default::default()
    };
    let output = format(input, option.clone());
    assert_eq!(output, expect);
    // 冪等性: 整形結果をもう一度整形しても変わらない
    assert_eq!(format(&output, option), output, "not idempotent");
}

#[test]
fn test_unicode() {
    let text = r#"

document(||)'<
+section{ section }<
+p {日本語}
>>"#;

    let expect = r#"document (||) '<
    +section { section } <
        +p { 日本語 }
    >
>
"#;
    test_tmpl(text, expect)
}
