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
