//! 新しい printer (`format_pretty`) が旧実装と同じ出力になることを実ファイルで確認する。
use crate::{format, format_pretty};
use lspower::lsp::FormattingOptions;

fn option() -> FormattingOptions {
    FormattingOptions {
        tab_size: 4,
        insert_spaces: true,
        ..Default::default()
    }
}

fn assert_same(name: &str, input: &str) {
    let old = format(input, option());
    let new = format_pretty(input, option());
    assert_eq!(old, new, "{name}: format_pretty differs from format");
    // 冪等性: 整形結果をもう一度整形しても変わらない
    assert_eq!(
        new,
        format_pretty(&new, option()),
        "{name}: format_pretty is not idempotent"
    );
}

#[test]
fn same_output_for_examples() {
    assert_same("ex1", include_str!("../../examples/ex1.saty"));
    assert_same("ex2", include_str!("../../examples/ex2.saty"));
    assert_same("ex3", include_str!("../../examples/ex3.saty"));
    assert_same("draft", include_str!("../../doc/draft.saty"));
    assert_same("doc", include_str!("../../doc/doc.saty"));
}

#[test]
fn comments_between_headers() {
    let input = "@import: a\n% c1\n@require: b % c2\n\n% c3\ndocument(||)'<>";
    let expect = "@import: a\n% c1\n@require: b % c2\n\n% c3\ndocument(||)'<>\n";
    assert_eq!(format_pretty(input, option()), expect);
}
