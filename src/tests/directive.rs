//! `% satysfi-fmt: off / on` による整形の制御 (clang-format の `// clang-format off` / `on` と同じ)
use super::{test_tmpl, test_width};

#[test]
fn lines_between_off_and_on_are_kept_as_is() {
    // 空行も含めて元のまま。範囲の外は整形される
    let text = r##"let    a   =   1
% satysfi-fmt: off
let    b   =   2


   let    c   =   3
% satysfi-fmt: on
let    d   =   4
"##;
    let expect = r##"let a = 1
% satysfi-fmt: off
let    b   =   2


   let    c   =   3
% satysfi-fmt: on
let d = 4
"##;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_comments_themselves_are_indented() {
    let text = r##"module M = struct
let    a   =   1
        % satysfi-fmt: off
let    b   =   2
            % satysfi-fmt: on
let    c   =   3
end
"##;
    let expect = r##"module M = struct
    let a = 1
    % satysfi-fmt: off
let    b   =   2
    % satysfi-fmt: on
    let c = 3
end
"##;
    test_tmpl(text, expect);
}

#[test]
fn off_without_on_continues_to_the_end_of_the_file() {
    // `struct ... end` の外にも及ぶ
    let text = r##"module M = struct
  % satysfi-fmt: off
  let    b   =   2
end
let    e   =   5"##;
    let expect = r##"module M = struct
    % satysfi-fmt: off
  let    b   =   2
end
let    e   =   5
"##;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_in_the_middle_of_a_list() {
    let text = r##"let x = f [1;
% satysfi-fmt: off
      2;    3;
             4;
% satysfi-fmt: on
      5;    6]
let    y   =   1
"##;
    let expect = r##"let x = f [
    1;
    % satysfi-fmt: off
      2;    3;
             4;
    % satysfi-fmt: on
    5;
    6;
]
let y = 1
"##;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_in_the_middle_of_a_record() {
    let text = r##"let r = (|
  a = 1;
  % satysfi-fmt: off
  b   =   [1;2];
  % satysfi-fmt: on
  c   =   [1;2]
|)
"##;
    let expect = r##"let r = (|
    a = 1;
    % satysfi-fmt: off
  b   =   [1;2];
    % satysfi-fmt: on
    c = [1; 2];
|)
"##;
    test_tmpl(text, expect);
}

#[test]
fn trailing_off_and_on() {
    // 行末の off: その行は整形し、次の行から元のまま
    // 行末の on: その行の on より前は元のまま、次の行から整形する
    let text = r##"let    a   =   1 % satysfi-fmt: off
let    b   =   2
      let    c   =   3   % satysfi-fmt: on
let    d   =   4
"##;
    let expect = r##"let a = 1 % satysfi-fmt: off
let    b   =   2
      let    c   =   3 % satysfi-fmt: on
let d = 4
"##;
    test_tmpl(text, expect);
}

#[test]
fn comments_between_off_and_on_are_kept_as_is() {
    let text = r##"% satysfi-fmt: off
let    a   =   1   %no space
%%   double
% satysfi-fmt: on
let    b   =   2   %c
"##;
    let expect = r##"% satysfi-fmt: off
let    a   =   1   %no space
%%   double
% satysfi-fmt: on
let b = 2 % c
"##;
    test_tmpl(text, expect);
}

#[test]
fn reason_may_follow_a_colon() {
    let text = r##"let    a   =   1
% satysfi-fmt: off: aligned table
let    b   =   2
% satysfi-fmt: on: back to normal
let    c   =   3
"##;
    let expect = r##"let a = 1
% satysfi-fmt: off: aligned table
let    b   =   2
% satysfi-fmt: on: back to normal
let c = 3
"##;
    test_tmpl(text, expect);
}

#[test]
fn second_off_before_on_is_ignored() {
    let text = r##"% satysfi-fmt: off
let    a   =   1
% satysfi-fmt: off
let    b   =   2
% satysfi-fmt: on
let    c   =   3
"##;
    let expect = r##"% satysfi-fmt: off
let    a   =   1
% satysfi-fmt: off
let    b   =   2
% satysfi-fmt: on
let c = 3
"##;
    test_tmpl(text, expect);
}

#[test]
fn on_without_off_is_ignored() {
    let text = "% satysfi-fmt: on\nlet   x   =   1\n";
    let expect = "% satysfi-fmt: on\nlet x = 1\n";
    test_tmpl(text, expect);
}

#[test]
fn unknown_directive_is_just_a_comment() {
    let text = "% satysfi-fmt: unknown\nlet   x   =   1\n";
    let expect = "% satysfi-fmt: unknown\nlet x = 1\n";
    test_tmpl(text, expect);
}

#[test]
fn directive_without_spaces_is_recognized() {
    let text = "%satysfi-fmt:off\nlet   x   =   1\n%satysfi-fmt:on\nlet   y   =   1\n";
    let expect = "% satysfi-fmt:off\nlet   x   =   1\n% satysfi-fmt:on\nlet y = 1\n";
    test_tmpl(text, expect);
}

#[test]
fn skip_is_not_supported_and_is_just_a_comment() {
    let text = "% satysfi-fmt: skip\nlet   x   =   [1;2] % satysfi-fmt: skip\nlet   y   =   1\n";
    let expect = "% satysfi-fmt: skip\nlet x = [1; 2] % satysfi-fmt: skip\nlet y = 1\n";
    test_tmpl(text, expect);
}

#[test]
fn region_is_not_wrapped_by_the_line_width() {
    let text = "% satysfi-fmt: off\nlet x = [aaaaaaaa;   bbbbbbbb;   cccccccc]\n% satysfi-fmt: on\nlet y = [aaaaaaaa; bbbbbbbb; cccccccc]\n";
    let expect = "% satysfi-fmt: off\nlet x = [aaaaaaaa;   bbbbbbbb;   cccccccc]\n% satysfi-fmt: on\nlet y = [\n    aaaaaaaa;\n    bbbbbbbb;\n    cccccccc;\n]\n";
    test_width(text, 20, expect);
}

#[test]
fn off_and_on_in_horizontal_mode() {
    let text = r##"document(||)'<
  +p {
    x   y
    % satysfi-fmt: off
    \emph  {  y  }   z
    % satysfi-fmt: on
    \emph  {  z  }
  }
>
"##;
    let expect = r##"document (||) '<
    +p {
        x y
        % satysfi-fmt: off
    \emph  {  y  }   z
        % satysfi-fmt: on
        \emph { z }
    }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_in_a_vertical_area() {
    let text = r##"document(||)'<
  +p{ a }
  % satysfi-fmt: off
  +p   {  b  }
  +p   {  c  }
  % satysfi-fmt: on
  +p   {  d  }
>
"##;
    let expect = r##"document (||) '<
    +p { a }
    % satysfi-fmt: off
  +p   {  b  }
  +p   {  c  }
    % satysfi-fmt: on
    +p { d }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_around_a_header_line() {
    let text = "@require: a\n% satysfi-fmt: off\n@import:   spaced\n% satysfi-fmt: on\n@import:   b\ndocument(||)'<>\n";
    let expect = "@require: a\n% satysfi-fmt: off\n@import:   spaced\n% satysfi-fmt: on\n@import: b\n\ndocument (||) '<>\n";
    test_tmpl(text, expect);
}

#[test]
fn off_in_the_preamble_reaches_the_document_body() {
    // `on` がなければファイルの最後まで
    let text = "let x = [1;   2]\n% satysfi-fmt: off\nlet y = [1;   2]\nin\ndocument (||) '<\n  +p{   abc }\n>\n";
    let expect = "let x = [1; 2]\n% satysfi-fmt: off\nlet y = [1;   2]\nin\ndocument (||) '<\n  +p{   abc }\n>\n";
    test_tmpl(text, expect);
}

// ---- 構文の途中に書いた off / on ----
// 整形でコメントの位置が変わり、範囲を正確には戻せないときは、
// off と on を含む同じ階層のノードの並びを丸ごと元のまま出力する (エラーにならない)。

#[test]
fn off_inside_a_statement_keeps_the_statement_as_is() {
    let text = "let-rec f x % satysfi-fmt: off\n = 1\nlet   y   =   [1;2]\n% satysfi-fmt: on\nlet   z   =   [1;2]\n";
    let expect = "let-rec f x % satysfi-fmt: off\n = 1\nlet   y   =   [1;2]\n% satysfi-fmt: on\nlet z = [1; 2]\n";
    test_tmpl(text, expect);
}

#[test]
fn off_before_a_name_does_not_duplicate_the_comment() {
    // 整形でコメントが文の前に動くが、元の文を戻すときに動いた方を取り除くので、コメントは 1 つだけ残る
    let text = "let-rec % satysfi-fmt: off\n f   x = 1\nlet   y   =   [1;2]\n% satysfi-fmt: on\nlet   z   =   [1;2]\n";
    let expect = "let-rec % satysfi-fmt: off\n f   x = 1\nlet   y   =   [1;2]\n% satysfi-fmt: on\nlet z = [1; 2]\n";
    test_tmpl(text, expect);
}

#[test]
fn off_after_equal_keeps_the_following_nodes() {
    let text = "let   a   =   [1;2]\nlet x = % satysfi-fmt: off\n   [1;2]\nlet   y   =   [1;2]\n% satysfi-fmt: on\nlet   z   =   [1;2]\n";
    let expect = "let a = [1; 2]\nlet x =\n    % satysfi-fmt: off\n   [1;2]\nlet   y   =   [1;2]\n% satysfi-fmt: on\nlet z = [1; 2]\n";
    test_tmpl(text, expect);
}

#[test]
fn off_in_a_type_definition_without_on_keeps_the_rest_of_the_file() {
    let text = "type t = % satysfi-fmt: off\n   A   |   B\nlet   z   =   [1;2]\n";
    let expect = "type t = % satysfi-fmt: off\n   A   |   B\nlet   z   =   [1;2]\n";
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_inside_one_list_keep_the_elements_between_them() {
    let text = "let   a   =   [1;2]\nlet x = [1;   2; % satysfi-fmt: off\n   3;   4; % satysfi-fmt: on\n 5]\nlet   z   =   [1;2]\n";
    let expect = "let a = [1; 2]\nlet x = [\n    1;\n    2; % satysfi-fmt: off\n   3;   4; % satysfi-fmt: on\n    5;\n]\nlet z = [1; 2]\n";
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_in_different_statements_widen_to_the_enclosing_statements() {
    // off を含む文 (リスト) と、on を含む文を含めて、間の文はすべて元のまま
    let text = "let   a   =   [1;2]\nlet x = [1;   2; % satysfi-fmt: off\n   3;   4]\nlet   y   =   [1;2]\nlet w = [5;   6; % satysfi-fmt: on\n 7]\nlet   z   =   [1;2]\n";
    let expect = "let a = [1; 2]\nlet x = [\n    1;\n    2; % satysfi-fmt: off\n   3;   4]\nlet   y   =   [1;2]\nlet w = [5;   6; % satysfi-fmt: on\n    7;\n]\nlet z = [1; 2]\n";
    test_tmpl(text, expect);
}

#[test]
fn format_never_returns_an_error_for_off_and_on() {
    use crate::{format, FormatConfig};
    // 構文エラー以外ではエラーにならない
    for text in [
        "let-rec % satysfi-fmt: off\n f x = 1\n",
        "let x = (1, % satysfi-fmt: off\n 2)\n",
        "type t = A % satysfi-fmt: on\n | B\n",
        "% satysfi-fmt: on\n% satysfi-fmt: off\nlet x = 1\n",
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
    let text = "let-rec f x = g x% satysfi-fmt: off\n\nand% satysfi-fmt: on\n g y   =   y\nlet   z   =   [1;2]\n";
    let expect = "let-rec f x = g x% satysfi-fmt: off\n\nand% satysfi-fmt: on\n g y   =   y\nlet z = [1; 2]\n";
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_around_a_symbol_keep_their_position() {
    let text = "type 'a t%satysfi-fmt: off\n =% satysfi-fmt: on\n A   |   B\nlet   z   =   [1;2]\n";
    let expect = "type 'a t%satysfi-fmt: off\n =% satysfi-fmt: on\n A   |   B\nlet z = [1; 2]\n";
    test_tmpl(text, expect);
}

#[test]
fn off_and_on_never_move_relative_to_the_code() {
    use super::directives_keep_their_position;
    use crate::{format, FormatConfig};
    for text in [
        "let-rec f x % satysfi-fmt: off\n = 1\nlet   y   =   [1;2]\n% satysfi-fmt: on\nlet   z   =   [1;2]\n",
        "let-rec % satysfi-fmt: off\n f   x = 1\nlet   y   =   [1;2]\n% satysfi-fmt: on\nlet   z   =   [1;2]\n",
        "type t = % satysfi-fmt: off\n   A   |   B\nlet   z   =   [1;2]\n",
        "let x = (1, % satysfi-fmt: off\n 2, % satysfi-fmt: on\n 3)\n",
        "let x = f a % satysfi-fmt: off\n b % satysfi-fmt: on\n c\n",
        "let x = if a % satysfi-fmt: off\n then b % satysfi-fmt: on\n else c\n",
    ] {
        let output = format(text, &FormatConfig::default()).unwrap();
        assert!(
            directives_keep_their_position(text, &output),
            "off / on moved:\n{text}\n--- output\n{output}"
        );
        // 2 回目の整形でも変わらない
        assert_eq!(format(&output, &FormatConfig::default()).unwrap(), output);
    }
}
