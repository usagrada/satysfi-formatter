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

/// 空白と `;` を除いた文字列。整形で並びが変わっていないことの確認に使う。
fn tokens(text: &str) -> String {
    let text: String = text
        .chars()
        .filter(|c| !c.is_whitespace() && *c != ';')
        .collect();
    // 型定義の先頭の `|` は省略できる
    text.replace("=|", "=")
}

/// `%` から行末までのコメントの一覧 (空白を除く)
fn comments(text: &str) -> Vec<String> {
    let mut result = vec![];
    for line in text.lines() {
        let mut escaped = false;
        let mut in_string = false;
        for (i, c) in line.char_indices() {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '`' {
                in_string = !in_string;
            } else if c == '%' && !in_string {
                result.push(line[i..].chars().filter(|c| !c.is_whitespace()).collect());
                break;
            }
        }
    }
    result
}


/// どんな行幅で整形しても成り立つ性質を確認する。
/// * 整形結果をもう一度整形しても変わらない
/// * 空白と `;` を除いたトークン列が変わらない
/// * コメントが増減しない
pub(crate) fn check_invariants(name: &str, input: &str) {
    for width in [20, 40, 80, 200] {
        let mut option = FormattingOptions {
            tab_size: 4,
            insert_spaces: true,
            ..Default::default()
        };
        option.properties.insert(
            "lineWidth".to_string(),
            lspower::lsp::FormattingProperty::Number(width),
        );
        let output = format(input, option.clone());
        assert_eq!(
            output,
            format(&output, option),
            "{name}@{width}: not idempotent"
        );
        assert_eq!(tokens(&output), tokens(input), "{name}@{width}: tokens changed");
        assert_eq!(
            comments(&output),
            comments(input),
            "{name}@{width}: comments changed"
        );
    }
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
