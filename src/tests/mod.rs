use crate::{format, format_pretty};
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
    // 新しい printer も同じ出力になること
    let output = format_pretty(input, option);
    assert_eq!(output, expect, "format_pretty");
}

#[test]
fn test_unicode() {
    let text = r#"

document(||)'<
+section{ section }<
+p {日本語}
>>"#;

    let expect = r#"document(||)'<
    +section { section } <
        +p { 日本語 }
    >
>
"#;
    test_tmpl(text, expect)
}
