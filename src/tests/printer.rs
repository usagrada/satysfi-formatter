use crate::format;
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

use super::check_invariants;

// 構文を一通り含む入力
const REC: &str = r##"@require: stdjareport

let-rec f x = g x
and g y = y
let-block +p it = '<>
let-inline \m x = {}
type 'a t = A | B of int constraint 'a :: (| x : int |)
and u = int
module M : sig
  type t
  val f : int -> int constraint 'a :: (| |)
end = struct
  let f x = x
end
in
let-rec h x = 1 | y = 2 in
let a = fun x -> Some x in
document (| title = {a} |) '<>
"##;

const COMMENTS: &str = r##"% license header
@require: stdjareport % trailing in header
% between headers
@import: local

% before let
let x = 1 % trailing let
let y = % after equal
  2
let l = [
  1; % one
  % own line
  2;
  % dangling
]
let r = (| a = 1; % first
  b = 2 |)
let f x y = % comment in def
  x + y % after
let g x =
  match x with
  | 0 -> 1 % zero
  % own-line before arm
  | _ -> 2
let t = (1, % first
  2)
in

% before document
document (| title = {a}; % title
  author = {b} |) '<
  % in vertical
  +p { hello % comment in text
    world }
  +p { a } % trailing block
  % end of vertical
> % after document
"##;

const MODES: &str = r##"@require: stdjareport

module M : sig
  type 'a t
  val f : int -> int
  val (+++) : int -> int -> int
  direct \tabular : [inline-text list; (| l : int |)] inline-cmd
end = struct
  type 'a t = A | B of int
  and u = int
  let f x = x
  let-rec g x = h x
  and h y = y
  let-rec k | 0 = 1 | n = n
  let-inline \foo x = {#x;}
  let-block +bar it = '<+p{ #it; }>
  let-math \sq x = ${\sqrt{#x}}
  let-mutable m <- 0
  open Foo
end

let z = fun x -> (if x then [1; 2] else [3])
let w = (fun x -> x) 1
let s = `a % b`
let o = Some 1
let pt = <[ (0pt, 0pt) -- (1pt, 1pt) .. controls (1pt, 0pt) and (0pt, 1pt) .. (2pt, 2pt) -- cycle ]>
let u = f ?:1 ?* 2
let a = !x
let q = M.(f 1)
let p = not true
let neg = -1
in

document (| title = {a} |) '<
  +p {
    hello\bold{world}text \emph{ x } and \br; and ${a^2 + b_{i j} = \frac{1}{2}}.
    #embed;
  }
  +section{Sec}<
    +p{ a }
    +listing{
      * item1
        ** sub
      * item2
    }
  >
  +table (| a = 1 |) [[ {a}; {b} ]];
  +math (${x}) ?:(1) ?*;
  #blockembed;
>
"##;

#[test]
fn examples_are_idempotent_and_keep_tokens_and_comments() {
    let files = [
        ("ex1", include_str!("../../examples/ex1.saty")),
        ("ex2", include_str!("../../examples/ex2.saty")),
        ("ex3", include_str!("../../examples/ex3.saty")),
        ("draft", include_str!("../../doc/draft.saty")),
        ("doc", include_str!("../../doc/doc.saty")),
        ("rec", REC),
        ("comments", COMMENTS),
        ("modes", MODES),
    ];
    for (name, input) in files {
        check_invariants(name, input);
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

#[test]
fn skip_a_header_line() {
    let input = "@require: a\n% satysfi-fmt: skip\n@import:   spaced\ndocument(||)'<>";
    let expect = "@require: a\n% satysfi-fmt: skip\n@import:   spaced\n\ndocument (||) '<>\n";
    assert_eq!(fmt(input, 80), expect);
}

#[test]
fn skip_in_horizontal_and_vertical() {
    let input = "document(||)'<\n  % satysfi-fmt: skip\n  +p   {  a   b  }\n  +p   {  a   b  }\n  +p {\n    x\n    % satysfi-fmt: skip\n    \\emph  {  y  }\n    \\emph  {  z  }\n  }\n>";
    let expect = "document (||) '<\n    % satysfi-fmt: skip\n    +p   {  a   b  }\n    +p { a b }\n    +p {\n        x\n        % satysfi-fmt: skip\n        \\emph  {  y  }\n        \\emph { z }\n    }\n>\n";
    assert_eq!(fmt(input, 80), expect);
}

#[test]
fn skip_in_struct() {
    let input = "module M = struct\n  % satysfi-fmt: skip\n  let   x   =   1\n  let   y   =   2\nend\n";
    let expect = "module M = struct\n    % satysfi-fmt: skip\n    let   x   =   1\n    let y = 2\nend\n";
    assert_eq!(fmt(input, 80), expect);
}

#[test]
fn multiline_string_literal_is_not_reindented() {
    let input = "let s = `\n  a\n    b\n`\nlet t = [`x\n  y`; 1]\n";
    let output = fmt(input, 80);
    assert!(output.contains("`\n  a\n    b\n`"), "{output}");
    assert!(output.contains("`x\n  y`"), "{output}");
    assert_eq!(output, fmt(&output, 80));
}

#[test]
fn horizontal_text_keeps_adjacency() {
    let input = "document(||)'<+p{a\\emph{b}c \\emph{d} e}>";
    let expect = "document (||) '<\n    +p { a\\emph { b }c \\emph { d } e }\n>\n";
    assert_eq!(fmt(input, 80), expect);
    // 長くても 1 行の文章は途中で折り返さない
    let long = "document(||)'<+p{aaaaaaaaaa bbbbbbbbbb cccccccccc dddddddddd eeeeeeeeee ffffffffff}>";
    assert_eq!(fmt(long, 20).lines().count(), 3);
}
