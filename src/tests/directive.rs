//! `% satysfi-fmt: skip / off / on` による整形の制御
use super::{test_tmpl, test_width};

#[test]
fn off_without_on_continues_to_the_end_of_the_parent() {
    let text = "let a = [1;2]\n% satysfi-fmt: off\nlet   b   =   [1;2]\nlet   c   =   [1;2]\n";
    let expect = "let a = [1; 2]\n% satysfi-fmt: off\nlet   b   =   [1;2]\nlet   c   =   [1;2]\n";
    test_tmpl(text, expect);
}

#[test]
fn off_inside_a_module_ends_with_the_module() {
    let text = "module M = struct\n  % satysfi-fmt: off\n  let   x   =   1\nend\nlet   y   =   1\n";
    let expect =
        "module M = struct\n    % satysfi-fmt: off\n    let   x   =   1\nend\n\nlet y = 1\n";
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
    let text = "%satysfi-fmt:skip\nlet   x   =   1\nlet   y   =   1\n";
    let expect = "% satysfi-fmt:skip\nlet   x   =   1\nlet y = 1\n";
    test_tmpl(text, expect);
}

#[test]
fn skip_affects_only_the_next_node() {
    let text = "let a = 1\n% satysfi-fmt: skip\nlet   b   =   [1;2]\nlet   c   =   [1;2]\n";
    let expect = "let a = 1\n% satysfi-fmt: skip\nlet   b   =   [1;2]\nlet c = [1; 2]\n";
    test_tmpl(text, expect);
}

#[test]
fn skip_a_record_field() {
    let text =
        "let r = (|\n  a = 1;\n  % satysfi-fmt: skip\n  b   =   [1;2];\n  c   =   [1;2]\n|)\n";
    let expect = "let r = (|\n    a = 1;\n    % satysfi-fmt: skip\n    b   =   [1;2];\n    c = [1; 2];\n|)\n";
    test_tmpl(text, expect);
}

#[test]
fn skip_an_expression_keeps_the_layout_of_its_neighbours() {
    let text = "let x =\n  % satysfi-fmt: skip\n  f   a    b\n";
    let expect = "let x =\n    % satysfi-fmt: skip\n    f   a    b\n";
    test_tmpl(text, expect);
}

#[test]
fn skipped_multiline_text_keeps_its_own_indentation() {
    let text = "let t = 1\n% satysfi-fmt: skip\nlet m = [\n      1;   2;\n   3 ]\n";
    let expect = "let t = 1\n% satysfi-fmt: skip\nlet m = [\n      1;   2;\n   3 ]\n";
    test_tmpl(text, expect);
}

#[test]
fn skipped_node_is_not_wrapped_by_the_line_width() {
    let text = "% satysfi-fmt: skip\nlet x = [aaaaaaaa;   bbbbbbbb;   cccccccc]\nlet y = [aaaaaaaa; bbbbbbbb; cccccccc]\n";
    let expect = "% satysfi-fmt: skip\nlet x = [aaaaaaaa;   bbbbbbbb;   cccccccc]\nlet y = [\n    aaaaaaaa;\n    bbbbbbbb;\n    cccccccc;\n]\n";
    test_width(text, 20, expect);
}

#[test]
fn trailing_skip_applies_to_the_node_on_the_same_line() {
    let text = "let   x   =   1 % satysfi-fmt: skip\nlet   y   =   1\n";
    let expect = "let   x   =   1 % satysfi-fmt: skip\nlet y = 1\n";
    test_tmpl(text, expect);
}

#[test]
fn skip_in_vertical_and_horizontal_modes() {
    let text = "document(||)'<\n  % satysfi-fmt: skip\n  +p   {  a   b  }\n  +p   {  a   b  }\n  +p {\n    x\n    % satysfi-fmt: skip\n    \\emph  {  y  }\n    \\emph  {  z  }\n  }\n>\n";
    let expect = "document (||) '<\n    % satysfi-fmt: skip\n    +p   {  a   b  }\n    +p { a b }\n    +p {\n        x\n        % satysfi-fmt: skip\n        \\emph  {  y  }\n        \\emph { z }\n    }\n>\n";
    test_tmpl(text, expect);
}

#[test]
fn skip_a_header_line() {
    let text = "@require: a\n% satysfi-fmt: skip\n@import:   spaced\ndocument(||)'<>\n";
    let expect = "@require: a\n% satysfi-fmt: skip\n@import:   spaced\n\ndocument (||) '<>\n";
    test_tmpl(text, expect);
}

#[test]
fn off_covers_blocks_in_a_vertical_area() {
    let text = "document(||)'<\n  +p{ a }\n  % satysfi-fmt: off\n  +p   {  b  }\n  +p   {  c  }\n  % satysfi-fmt: on\n  +p   {  d  }\n>\n";
    let expect = "document (||) '<\n    +p { a }\n    % satysfi-fmt: off\n    +p   {  b  }\n    +p   {  c  }\n    % satysfi-fmt: on\n    +p { d }\n>\n";
    test_tmpl(text, expect);
}

#[test]
fn skip_before_a_bracketed_value_stays_on_its_own_line() {
    // `=` の行末に移ると、次の整形で「直前のノードの skip」になってしまう
    let text = "let x =\n  % satysfi-fmt: skip\n  [1;   2]\n";
    let expect = "let x =\n    % satysfi-fmt: skip\n    [1;   2]\n";
    test_tmpl(text, expect);
}

#[test]
fn trailing_skip_on_the_last_statement_skips_only_that_statement() {
    let text = "let x = [1;   2]\nlet y = (a ,  b) % satysfi-fmt: skip\nin\ndocument (||) '<>\n";
    let expect = "let x = [1; 2]\nlet y = (a ,  b) % satysfi-fmt: skip\nin\n\ndocument (||) '<>\n";
    test_tmpl(text, expect);
}

#[test]
fn off_in_the_preamble_does_not_reach_the_document_body() {
    let text = "let x = [1;   2]\n% satysfi-fmt: off\nlet y = [1;   2]\nin\ndocument (||) '<\n  +p{   abc }\n>\n";
    let expect = "let x = [1; 2]\n% satysfi-fmt: off\nlet y = [1;   2]\nin\n\ndocument (||) '<\n    +p { abc }\n>\n";
    test_tmpl(text, expect);
}
