use crate::{format, format_legacy};
use lspower::lsp::{FormattingOptions, FormattingProperty};

fn option_with_width(width: i32) -> FormattingOptions {
    let mut option = FormattingOptions {
        tab_size: 4,
        insert_spaces: true,
        ..Default::default()
    };
    option
        .properties
        .insert("lineWidth".to_string(), FormattingProperty::Number(width));
    option
}

fn fmt(input: &str, width: i32) -> String {
    format(input, option_with_width(width))
}

/// 空白と `;` を除いた文字列。整形で並びが変わっていないことの確認に使う。
fn tokens(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && *c != ';')
        .collect()
}

#[test]
fn examples_are_idempotent_and_keep_tokens() {
    let files = [
        ("ex1", include_str!("../../examples/ex1.saty")),
        ("ex2", include_str!("../../examples/ex2.saty")),
        ("ex3", include_str!("../../examples/ex3.saty")),
        ("draft", include_str!("../../doc/draft.saty")),
        ("doc", include_str!("../../doc/doc.saty")),
    ];
    for (name, input) in files {
        for width in [20, 80, 200] {
            let output = fmt(input, width);
            assert_eq!(
                output,
                fmt(&output, width),
                "{name}@{width}: not idempotent"
            );
            assert_eq!(
                tokens(&output),
                tokens(&format_legacy(input, option_with_width(width))),
                "{name}@{width}: tokens changed"
            );
        }
    }
}

#[test]
fn list_fits_in_one_line() {
    let input = "let x = [1; 2; 3;]\n";
    assert_eq!(fmt(input, 80), "let x = [1; 2; 3]\n");
}

#[test]
fn list_breaks_when_too_long() {
    let input = "let x = [aaaaaaaa; bbbbbbbb; cccccccc]\n";
    let expect = "let x = [\n    aaaaaaaa;\n    bbbbbbbb;\n    cccccccc;\n]\n";
    assert_eq!(fmt(input, 20), expect);
}

#[test]
fn empty_list_and_record() {
    assert_eq!(fmt("let x = [ ]\nlet y = (| |)\n", 80), "let x = []\nlet y = (||)\n");
}

#[test]
fn record_fits_in_one_line() {
    let input = "let r = (| a = 1; b = 2; |)\n";
    assert_eq!(fmt(input, 80), "let r = (|a = 1; b = 2|)\n");
}

#[test]
fn record_breaks_when_too_long() {
    let input = "let r = (| aaaa = 1; bbbb = 2 |)\n";
    let expect = "let r = (|\n    aaaa = 1;\n    bbbb = 2;\n|)\n";
    assert_eq!(fmt(input, 16), expect);
}

#[test]
fn nested_list_breaks_only_outer_when_needed() {
    let input = "let x = [[1; 2]; [3; 4]]\n";
    let expect = "let x = [\n    [1; 2];\n    [3; 4];\n]\n";
    assert_eq!(fmt(input, 18), expect);
}

#[test]
fn application_breaks_arguments() {
    let input = "let y = f aaaaaaaa bbbbbbbb cccccccc\n";
    let expect = "let y =\n    f\n        aaaaaaaa\n        bbbbbbbb\n        cccccccc\n";
    assert_eq!(fmt(input, 20), expect);
    assert_eq!(fmt(input, 80), "let y = f aaaaaaaa bbbbbbbb cccccccc\n");
}

#[test]
fn application_hugs_last_list() {
    let input = "let y = vconcat [aaaaaaaa; bbbbbbbb; cccccccc]\n";
    let expect = "let y = vconcat [\n    aaaaaaaa;\n    bbbbbbbb;\n    cccccccc;\n]\n";
    assert_eq!(fmt(input, 30), expect);
}

#[test]
fn operators_break_before_operator() {
    let input = "let y = aaaaaaaa |> bbbbbbbb |> cccccccc\n";
    assert_eq!(fmt(input, 80), input);
    let expect = "let y =\n    aaaaaaaa\n        |> bbbbbbbb\n        |> cccccccc\n";
    assert_eq!(fmt(input, 20), expect);
}

#[test]
fn comments_force_a_line_break() {
    let input = "let x = [a; % first\n  b]\n";
    let expect = "let x = [\n    a; % first\n    b;\n]\n";
    assert_eq!(fmt(input, 80), expect);
}

#[test]
fn own_line_comment_in_list() {
    let input = "let x = [\n  % head\n  a;\n  b\n  % tail\n]\n";
    let expect = "let x = [\n    % head\n    a;\n    b;\n    % tail\n]\n";
    assert_eq!(fmt(input, 80), expect);
}

#[test]
fn skip_next_node() {
    let input = "% satysfi-fmt: skip\nlet   x   =   [1;2]\nlet   y   =   [1;2]\n";
    let expect = "let   x   =   [1;2]\nlet y = [1; 2]\n";
    // コメント自体は残る
    assert_eq!(fmt(input, 80), format!("% satysfi-fmt: skip\n{expect}"));
}

#[test]
fn skip_with_trailing_comment() {
    let input = "let   x   =   [1;2] % satysfi-fmt: skip\nlet   y   =   [1;2]\n";
    let expect = "let   x   =   [1;2] % satysfi-fmt: skip\nlet y = [1; 2]\n";
    assert_eq!(fmt(input, 80), expect);
}

#[test]
fn off_and_on() {
    let input = "let a = [1;2]\n% satysfi-fmt: off\nlet   b   =   [1;2]\nlet   c   =   [1;2]\n% satysfi-fmt: on\nlet d = [1;2]\n";
    let expect = "let a = [1; 2]\n% satysfi-fmt: off\nlet   b   =   [1;2]\nlet   c   =   [1;2]\n% satysfi-fmt: on\nlet d = [1; 2]\n";
    assert_eq!(fmt(input, 80), expect);
}

#[test]
fn skip_keeps_multiline_text_as_is() {
    let input = "let a = 1\n% satysfi-fmt: skip\nlet m = [\n      1;   2;\n   3 ]\nlet b = 2\n";
    let expect = "let a = 1\n% satysfi-fmt: skip\nlet m = [\n      1;   2;\n   3 ]\nlet b = 2\n";
    assert_eq!(fmt(input, 80), expect);
}

#[test]
fn skip_inside_a_list_element() {
    let input = "let x = [\n  a;\n  % satysfi-fmt: skip\n  f   b   c;\n  d\n]\n";
    let expect = "let x = [\n    a;\n    % satysfi-fmt: skip\n    f   b   c;\n    d;\n]\n";
    assert_eq!(fmt(input, 80), expect);
}

#[test]
fn comments_between_headers() {
    let input = "@import: a\n% c1\n@require: b % c2\n\n% c3\ndocument(||)'<>";
    let expect = "@import: a\n% c1\n@require: b % c2\n\n% c3\ndocument (||) '<>\n";
    assert_eq!(fmt(input, 80), expect);
}

#[test]
fn blank_lines_between_statements_are_kept_up_to_one() {
    let input = "let a = 1\n\n\n\nlet b = 2\nlet c = 3\n";
    assert_eq!(fmt(input, 80), "let a = 1\n\nlet b = 2\nlet c = 3\n");
}

#[test]
fn skip_applies_to_first_statement() {
    // 先頭の文の直前の directive も、その文だけにかかる
    let input = "% satysfi-fmt: skip\nlet   x   =   [1;2]\nlet   y   =   [1;2]\n";
    assert_eq!(
        fmt(input, 80),
        "% satysfi-fmt: skip\nlet   x   =   [1;2]\nlet y = [1; 2]\n"
    );
}
