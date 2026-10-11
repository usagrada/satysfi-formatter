//! `% satysfi-fmt: off / on` による整形の制御 (clang-format の `// clang-format off` / `on` と同じ)
use super::{test_tmpl, test_width};

#[test]
fn lines_between_off_and_on_are_kept_as_is() {
    // 空行も含めて元のまま。範囲の外は整形される
    let text = r#"let    a   =   1
% satysfi-fmt: off
let    b   =   2


   let    c   =   3
% satysfi-fmt: on
let    d   =   4
"#;
    let expect = r#"let a = 1
% satysfi-fmt: off
let    b   =   2


   let    c   =   3
% satysfi-fmt: on
let d = 4
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_comments_themselves_are_indented() {
    let text = r#"module M = struct
let    a   =   1
        % satysfi-fmt: off
let    b   =   2
            % satysfi-fmt: on
let    c   =   3
end
"#;
    let expect = r#"module M = struct
    let a = 1
    % satysfi-fmt: off
let    b   =   2
    % satysfi-fmt: on
    let c = 3
end
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_without_on_continues_to_the_end_of_the_file() {
    // `struct ... end` の外にも及ぶ
    let text = r#"module M = struct
  % satysfi-fmt: off
  let    b   =   2
end
let    e   =   5"#;
    let expect = r#"module M = struct
    % satysfi-fmt: off
  let    b   =   2
end
let    e   =   5
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_in_the_middle_of_a_list() {
    let text = r#"let x = f [1;
% satysfi-fmt: off
      2;    3;
             4;
% satysfi-fmt: on
      5;    6]
let    y   =   1
"#;
    let expect = r#"let x = f [
    1;
    % satysfi-fmt: off
      2;    3;
             4;
    % satysfi-fmt: on
    5;
    6;
]
let y = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_in_the_middle_of_a_record() {
    let text = r#"let r = (|
  a = 1;
  % satysfi-fmt: off
  b   =   [1;2];
  % satysfi-fmt: on
  c   =   [1;2]
|)
"#;
    let expect = r#"let r = (|
    a = 1;
    % satysfi-fmt: off
  b   =   [1;2];
    % satysfi-fmt: on
    c = [1; 2];
|)
"#;
    test_tmpl(text, expect);
}

#[test]
fn trailing_off_and_on() {
    // 行末の off: その行は整形し、次の行から元のまま
    // 行末の on: その行の on より前は元のまま、次の行から整形する
    let text = r#"let    a   =   1 % satysfi-fmt: off
let    b   =   2
      let    c   =   3   % satysfi-fmt: on
let    d   =   4
"#;
    let expect = r#"let a = 1 % satysfi-fmt: off
let    b   =   2
      let    c   =   3 % satysfi-fmt: on
let d = 4
"#;
    test_tmpl(text, expect);
}

#[test]
fn comments_between_off_and_on_are_kept_as_is() {
    let text = r#"% satysfi-fmt: off
let    a   =   1   %no space
%%   double
% satysfi-fmt: on
let    b   =   2   %c
"#;
    let expect = r#"% satysfi-fmt: off
let    a   =   1   %no space
%%   double
% satysfi-fmt: on
let b = 2 % c
"#;
    test_tmpl(text, expect);
}

#[test]
fn reason_may_follow_a_colon() {
    let text = r#"let    a   =   1
% satysfi-fmt: off: aligned table
let    b   =   2
% satysfi-fmt: on: back to normal
let    c   =   3
"#;
    let expect = r#"let a = 1
% satysfi-fmt: off: aligned table
let    b   =   2
% satysfi-fmt: on: back to normal
let c = 3
"#;
    test_tmpl(text, expect);
}

#[test]
fn second_off_before_on_is_ignored() {
    let text = r#"% satysfi-fmt: off
let    a   =   1
% satysfi-fmt: off
let    b   =   2
% satysfi-fmt: on
let    c   =   3
"#;
    let expect = r#"% satysfi-fmt: off
let    a   =   1
% satysfi-fmt: off
let    b   =   2
% satysfi-fmt: on
let c = 3
"#;
    test_tmpl(text, expect);
}

#[test]
fn on_without_off_is_ignored() {
    let text = r#"% satysfi-fmt: on
let   x   =   1
"#;
    let expect = r#"% satysfi-fmt: on
let x = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn unknown_directive_is_just_a_comment() {
    let text = r#"% satysfi-fmt: unknown
let   x   =   1
"#;
    let expect = r#"% satysfi-fmt: unknown
let x = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn directive_without_spaces_is_recognized() {
    let text = r#"%satysfi-fmt:off
let   x   =   1
%satysfi-fmt:on
let   y   =   1
"#;
    let expect = r#"% satysfi-fmt:off
let   x   =   1
% satysfi-fmt:on
let y = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn skip_is_not_supported_and_is_just_a_comment() {
    let text = r#"% satysfi-fmt: skip
let   x   =   [1;2] % satysfi-fmt: skip
let   y   =   1
"#;
    let expect = r#"% satysfi-fmt: skip
let x = [1; 2] % satysfi-fmt: skip
let y = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn region_is_not_wrapped_by_the_line_width() {
    let text = r#"% satysfi-fmt: off
let x = [aaaaaaaa;   bbbbbbbb;   cccccccc]
% satysfi-fmt: on
let y = [aaaaaaaa; bbbbbbbb; cccccccc]
"#;
    let expect = r#"% satysfi-fmt: off
let x = [aaaaaaaa;   bbbbbbbb;   cccccccc]
% satysfi-fmt: on
let y = [
    aaaaaaaa;
    bbbbbbbb;
    cccccccc;
]
"#;
    test_width(text, 20, expect);
}

#[test]
fn off_and_on_in_horizontal_mode() {
    let text = r#"document(||)'<
  +p {
    x   y
    % satysfi-fmt: off
    \emph  {  y  }   z
    % satysfi-fmt: on
    \emph  {  z  }
  }
>
"#;
    let expect = r#"document (||) '<
    +p {
        x y
        % satysfi-fmt: off
    \emph  {  y  }   z
        % satysfi-fmt: on
        \emph { z }
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_in_a_vertical_area() {
    let text = r#"document(||)'<
  +p{ a }
  % satysfi-fmt: off
  +p   {  b  }
  +p   {  c  }
  % satysfi-fmt: on
  +p   {  d  }
>
"#;
    let expect = r#"document (||) '<
    +p { a }
    % satysfi-fmt: off
  +p   {  b  }
  +p   {  c  }
    % satysfi-fmt: on
    +p { d }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_around_a_header_line() {
    let text = r#"@require: a
% satysfi-fmt: off
@import:   spaced
% satysfi-fmt: on
@import:   b
document(||)'<>
"#;
    let expect = r#"@require: a
% satysfi-fmt: off
@import:   spaced
% satysfi-fmt: on
@import: b

document (||) '<>
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_in_the_preamble_reaches_the_document_body() {
    // `on` がなければファイルの最後まで
    let text = r#"let x = [1;   2]
% satysfi-fmt: off
let y = [1;   2]
in
document (||) '<
  +p{   abc }
>
"#;
    let expect = r#"let x = [1; 2]
% satysfi-fmt: off
let y = [1;   2]
in
document (||) '<
  +p{   abc }
>
"#;
    test_tmpl(text, expect);
}

// ---- 構文の途中に書いた off / on ----
// 整形でコメントの位置が変わり、範囲を正確には戻せないときは、
// off と on を含む同じ階層のノードの並びを丸ごと元のまま出力する (エラーにならない)。

#[test]
fn off_inside_a_statement_keeps_the_statement_as_is() {
    let text = r#"let-rec f x % satysfi-fmt: off
 = 1
let   y   =   [1;2]
% satysfi-fmt: on
let   z   =   [1;2]
"#;
    let expect = r#"let-rec f x % satysfi-fmt: off
 = 1
let   y   =   [1;2]
% satysfi-fmt: on
let z = [1; 2]
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_before_a_name_does_not_duplicate_the_comment() {
    // 整形でコメントが文の前に動くが、元の文を戻すときに動いた方を取り除くので、コメントは 1 つだけ残る
    let text = r#"let-rec % satysfi-fmt: off
 f   x = 1
let   y   =   [1;2]
% satysfi-fmt: on
let   z   =   [1;2]
"#;
    let expect = r#"let-rec % satysfi-fmt: off
 f   x = 1
let   y   =   [1;2]
% satysfi-fmt: on
let z = [1; 2]
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_after_equal_keeps_the_following_nodes() {
    let text = r#"let   a   =   [1;2]
let x = % satysfi-fmt: off
   [1;2]
let   y   =   [1;2]
% satysfi-fmt: on
let   z   =   [1;2]
"#;
    let expect = r#"let a = [1; 2]
let x =
    % satysfi-fmt: off
   [1;2]
let   y   =   [1;2]
% satysfi-fmt: on
let z = [1; 2]
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_in_a_type_definition_without_on_keeps_the_rest_of_the_file() {
    let text = r#"type t = % satysfi-fmt: off
   A   |   B
let   z   =   [1;2]
"#;
    let expect = r#"type t = % satysfi-fmt: off
   A   |   B
let   z   =   [1;2]
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_inside_one_list_keep_the_elements_between_them() {
    let text = r#"let   a   =   [1;2]
let x = [1;   2; % satysfi-fmt: off
   3;   4; % satysfi-fmt: on
 5]
let   z   =   [1;2]
"#;
    let expect = r#"let a = [1; 2]
let x = [
    1;
    2; % satysfi-fmt: off
   3;   4; % satysfi-fmt: on
    5;
]
let z = [1; 2]
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_in_different_statements_widen_to_the_enclosing_statements() {
    // off を含む文 (リスト) と、on を含む文を含めて、間の文はすべて元のまま
    let text = r#"let   a   =   [1;2]
let x = [1;   2; % satysfi-fmt: off
   3;   4]
let   y   =   [1;2]
let w = [5;   6; % satysfi-fmt: on
 7]
let   z   =   [1;2]
"#;
    let expect = r#"let a = [1; 2]
let x = [
    1;
    2; % satysfi-fmt: off
   3;   4]
let   y   =   [1;2]
let w = [5;   6; % satysfi-fmt: on
    7;
]
let z = [1; 2]
"#;
    test_tmpl(text, expect);
}

#[test]
fn format_never_returns_an_error_for_off_and_on() {
    use crate::{format, FormatConfig};
    // 構文エラー以外ではエラーにならない
    for text in [
        r#"let-rec % satysfi-fmt: off
 f x = 1
"#,
        r#"let x = (1, % satysfi-fmt: off
 2)
"#,
        r#"type t = A % satysfi-fmt: on
 | B
"#,
        r#"% satysfi-fmt: on
% satysfi-fmt: off
let x = 1
"#,
        "let x = 1 % satysfi-fmt: off",
    ] {
        assert!(
            format(text, &FormatConfig::default()).is_ok(),
            "failed: {text:?}"
        );
    }
}

#[test]
fn off_and_on_around_only_a_keyword_keep_their_position() {
    // off と on の間にノードがなく、`and` だけがある。整形で on が `and` の前に動かないよう、
    // 両方を含む `let-rec` の文を元のまま出力する
    let text = r#"let-rec f x = g x% satysfi-fmt: off

and% satysfi-fmt: on
 g y   =   y
let   z   =   [1;2]
"#;
    let expect = r#"let-rec f x = g x% satysfi-fmt: off

and% satysfi-fmt: on
 g y   =   y
let z = [1; 2]
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_around_a_symbol_keep_their_position() {
    let text = r#"type 'a t%satysfi-fmt: off
 =% satysfi-fmt: on
 A   |   B
let   z   =   [1;2]
"#;
    let expect = r#"type 'a t%satysfi-fmt: off
 =% satysfi-fmt: on
 A   |   B
let z = [1; 2]
"#;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_never_move_relative_to_the_code() {
    use super::directives_keep_their_position;
    use crate::{format, FormatConfig};
    for text in [
        r#"let-rec f x % satysfi-fmt: off
 = 1
let   y   =   [1;2]
% satysfi-fmt: on
let   z   =   [1;2]
"#,
        r#"let-rec % satysfi-fmt: off
 f   x = 1
let   y   =   [1;2]
% satysfi-fmt: on
let   z   =   [1;2]
"#,
        r#"type t = % satysfi-fmt: off
   A   |   B
let   z   =   [1;2]
"#,
        r#"let x = (1, % satysfi-fmt: off
 2, % satysfi-fmt: on
 3)
"#,
        r#"let x = f a % satysfi-fmt: off
 b % satysfi-fmt: on
 c
"#,
        r#"let x = if a % satysfi-fmt: off
 then b % satysfi-fmt: on
 else c
"#,
    ] {
        let output = format(text, &FormatConfig::default()).unwrap();
        assert!(
            directives_keep_their_position(text, &output),
            r#"off / on moved:
{text}
--- output
{output}"#
        );
        // 2 回目の整形でも変わらない
        assert_eq!(format(&output, &FormatConfig::default()).unwrap(), output);
    }
}

// ---- off / on として扱うコメントは `% satysfi-fmt: off` / `on` だけ ----

fn format_default(text: &str) -> String {
    crate::format(text, &crate::FormatConfig::default()).unwrap()
}

#[test]
fn ordinary_comments_do_not_stop_formatting() {
    for comment in [
        "% off",
        "% on",
        "% format: off",
        "% satysfi-fmt-off",
        "% satysfi-fmt off",
        "% satysfi-fmt:",
        "% SATYSFI-FMT: OFF",
        "% satysfi-fmt: offset",
        "% satysfi-fmt: off the table",
        "% turn satysfi-fmt: off here",
        "%% satysfi-fmt: off the table",
        "% satysfi-fmt: skip",
        "% satysfi-fmt: unknown",
    ] {
        let text = format!("let   a   =   [1;2]\n{comment}\nlet   b   =   [1;2]\n");
        let output = format_default(&text);
        // どちらの文も整形され、コメントは残る
        assert!(
            output.contains("let a = [1; 2]"),
            r#"{comment}
{output}"#
        );
        assert!(
            output.contains("let b = [1; 2]"),
            r#"{comment}
{output}"#
        );
        assert!(
            output.contains(comment.trim_end()) || output.contains(&comment.replacen('%', "% ", 1)),
            r#"{comment}
{output}"#
        );
    }
}

#[test]
fn only_the_directive_comments_stop_formatting() {
    for comment in [
        "% satysfi-fmt: off",
        "%satysfi-fmt:off",
        "%    satysfi-fmt:    off   ",
        "%% satysfi-fmt: off",
        "%%%satysfi-fmt:off",
        "% satysfi-fmt: off: aligned table",
    ] {
        let text = format!("let   a   =   [1;2]\n{comment}\nlet   b   =   [1;2]\n");
        let output = format_default(&text);
        assert!(
            output.contains("let a = [1; 2]"),
            r#"{comment}
{output}"#
        );
        assert!(
            output.contains("let   b   =   [1;2]"),
            r#"{comment}
{output}"#
        );
    }
}

#[test]
fn on_without_off_does_nothing() {
    let text = r#"let   a   =   [1;2]
% satysfi-fmt: on
let   b   =   [1;2]
"#;
    assert_eq!(
        format_default(text),
        r#"let a = [1; 2]
% satysfi-fmt: on
let b = [1; 2]
"#
    );
}

#[test]
fn directive_text_that_is_not_a_comment_is_ignored() {
    // 文字列、ヘッダ行、エスケープした `\%` の中は、コメントではない
    let text = r#"@require: a % satysfi-fmt: off
let   s   =   `% satysfi-fmt: off`
let   t   =   [1;2]
in
document(||)'<+p{a \% satysfi-fmt: off
}+p{  b  }>
"#;
    let output = format_default(text);
    assert!(output.contains("let t = [1; 2]"), "{output}");
    assert!(output.contains("+p { b }"), "{output}");
}

#[test]
fn a_directive_in_a_comment_after_code_on_the_same_line_is_a_directive() {
    // 行末に書いたコメントも、`% satysfi-fmt: off` の形なら有効
    let text = r#"let   a   =   1 % satysfi-fmt: off
let   b   =   [1;2]
% satysfi-fmt: on
let   c   =   [1;2]
"#;
    let output = format_default(text);
    assert!(output.contains("let   b   =   [1;2]"), "{output}");
    assert!(output.contains("let c = [1; 2]"), "{output}");
}
