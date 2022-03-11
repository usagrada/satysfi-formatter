use crate::format;

mod comment;
mod common;
mod horizontal_single;
mod let_block;
mod module;
mod space;

fn test_tmpl(input: &str, expect: &str) {
    let output = format(input);
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
