use crate::format;
use lspower::lsp::FormattingOptions;

mod comment;
mod comment_position;
mod common;
mod ctrl_stmt;
mod directive;
mod exprs;
mod horizontal_single;
mod let_block;
mod math;
mod math_mode;
mod module;
mod options;
mod patterns;
mod printer;
mod space;
mod stmts;
mod syntax;
mod text_mode;
mod types;

/// 行幅を指定して整形し、期待する出力と一致し、冪等であることを確認する。
fn test_width(input: &str, width: i32, expect: &str) {
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
    assert_eq!(output, expect, "width = {width}");
    assert_eq!(
        format(&output, option),
        output,
        "not idempotent (width = {width})"
    );
}

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

/// `%` から行末までのコメントの開始位置 (文字列やエスケープの中の `%` は除く)
fn comment_start(line: &str) -> Option<usize> {
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
            return Some(i);
        }
    }
    None
}

/// コメント・空白・`;` を除いた文字列。整形で並びが変わっていないことの確認に使う。
fn tokens(text: &str) -> String {
    let text: String = text
        .lines()
        .map(|line| &line[..comment_start(line).unwrap_or(line.len())])
        .flat_map(|line| line.chars())
        .filter(|c| !c.is_whitespace() && *c != ';')
        .collect();
    // 型定義の先頭の `|` は省略できる
    text.replace("=|", "=")
}

/// `%` から行末までのコメントの一覧 (空白を除く)
fn comments(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| comment_start(line).map(|i| &line[i..]))
        .map(|comment| comment.chars().filter(|c| !c.is_whitespace()).collect())
        .collect()
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
        assert_eq!(
            tokens(&output),
            tokens(input),
            "{name}@{width}: tokens changed"
        );
        assert_eq!(
            comments(&output),
            comments(input),
            "{name}@{width}: comments changed"
        );
    }
}

/// ソース中のあらゆる空白の位置にコメントを挿入して整形しても、
/// コメントが失われず、整形結果が冪等で、トークン列が変わらないことを確認する。
/// 挿入によってパースできなくなる位置は飛ばす。確認した位置の数を返す。
pub(crate) fn check_comment_insertion(name: &str, src: &str) -> usize {
    use satysfi_parser::{grammar, CstText};
    let option = FormattingOptions {
        tab_size: 4,
        insert_spaces: true,
        ..Default::default()
    };
    let mut checked = 0;
    let mut failures = vec![];
    let mut in_string = false;
    for (i, ch) in src.char_indices() {
        if ch == '`' {
            in_string = !in_string;
        }
        if in_string || !(ch == ' ' || ch == '\n') {
            continue;
        }
        // 既存のコメントの中には挿入しない
        let line_start = src[..i].rfind('\n').map_or(0, |k| k + 1);
        if src[line_start..i].contains('%') {
            continue;
        }
        let modified = format!("{}%cmt{}\n{}", &src[..i], checked, &src[i..]);
        if CstText::parse(&modified, grammar::program).is_err() {
            continue;
        }
        checked += 1;
        let output = format(&modified, option.clone());
        let mut expected_comments = comments(&modified);
        expected_comments.sort();
        let mut actual_comments = comments(&output);
        actual_comments.sort();
        let problem = if actual_comments != expected_comments {
            Some("comments changed")
        } else if format(&output, option.clone()) != output {
            Some("not idempotent")
        } else if tokens(&output) != tokens(&modified) {
            Some("tokens changed")
        } else {
            None
        };
        if let Some(problem) = problem {
            let context: String = src[..i]
                .chars()
                .rev()
                .take(24)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            failures.push(format!(
                "{name}: {problem} after {context:?}\n--- input\n{modified}\n--- output\n{output}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cases failed.\n{}\nfirst:\n{}",
        failures.len(),
        checked,
        failures
            .iter()
            .map(|f| f.lines().next().unwrap_or("").to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        failures.first().unwrap()
    );
    checked
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
