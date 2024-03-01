use lspower::lsp::FormattingOptions;

use crate::format;

mod comment;
mod common;
mod ctrl_stmt;
mod horizontal_single;
mod let_block;
mod math;
mod module;
// mod package;
mod space;

fn test_tmpl(input: &str, expect: &str) {
    let option = FormattingOptions {
        insert_spaces: true,
        tab_size: 4,
        ..Default::default()
    };
    let output = format(input, option);
    eprintln!("output\n=======\n{}\n=======", output);
    assert_eq!(output, expect);
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
