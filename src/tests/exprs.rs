//! 式の整形
use super::{test_tmpl, test_width};

#[test]
fn match_arms_are_split() {
    let text = r##"let g = match x with | A -> 1 | B y -> 2 | _ -> 3
"##;
    let expect = r##"let g =
    match x with
    | A -> 1
    | B y -> 2
    | _ -> 3
"##;
    test_tmpl(text, expect);
}

#[test]
fn match_arm_body_breaks() {
    let text = r##"let g = match x with | A -> aaaaaaaaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbbbbbbbbbbbb + ccccccccccccccccccccccccc | _ -> 3
"##;
    let expect = r##"let g =
    match x with
    | A ->
        aaaaaaaaaaaaaaaaaaaaaaaaa
            + bbbbbbbbbbbbbbbbbbbbbbbbbbbb
            + ccccccccccccccccccccccccc
    | _ -> 3
"##;
    test_width(text, 40, expect);
}

#[test]
fn if_in_one_line() {
    let text = r##"let g = if a then b else c
"##;
    let expect = r##"let g = if a then b else c
"##;
    test_tmpl(text, expect);
}

#[test]
fn if_breaks_when_too_long() {
    let text = r##"let g = if aaaaaaaaaaaaa then bbbbbbbbbbbbb else cccccccccccc
"##;
    let expect = r##"let g =
    if aaaaaaaaaaaaa then
        bbbbbbbbbbbbb
    else
        cccccccccccc
"##;
    test_width(text, 30, expect);
}

#[test]
fn else_if_chain() {
    let text = r##"let g = if a then b else if c then d else e
"##;
    let expect = r##"let g =
    if a then
        b
    else if c then
        d
    else
        e
"##;
    test_width(text, 20, expect);
}

#[test]
fn while_loop() {
    let text = r##"let g = while a do b <- 1
"##;
    let expect = r##"let g = while a do b <- 1
"##;
    test_tmpl(text, expect);
}

#[test]
fn lambda_in_one_line() {
    let text = r##"let g = fun x y -> x + y
"##;
    let expect = r##"let g = fun x y -> x + y
"##;
    test_tmpl(text, expect);
}

#[test]
fn lambda_body_breaks() {
    let text = r##"let g = fun x -> aaaaaaaaaaaaaaaaaaa + bbbbbbbbbbbbbbbbbb + ccccccccccccccccccccc
"##;
    let expect = r##"let g =
    fun x ->
        aaaaaaaaaaaaaaaaaaa
            + bbbbbbbbbbbbbbbbbb
            + ccccccccccccccccccccc
"##;
    test_width(text, 30, expect);
}

#[test]
fn lambda_in_parentheses_hugs() {
    let text = r##"let g = f (fun x -> x + 1) 2
"##;
    let expect = r##"let g = f (fun x -> x + 1) 2
"##;
    test_tmpl(text, expect);
}

#[test]
fn sequential() {
    let text = r##"let g = a <- 1 before b <- 2
"##;
    let expect = r##"let g = a <- 1 before b <- 2
"##;
    test_tmpl(text, expect);
}

#[test]
fn sequential_breaks_after_before() {
    let text = r##"let g = a <- 1 before b <- 2 before c
"##;
    let expect = r##"let g =
    a <-
        1 before
    b <-
        2 before
    c
"##;
    test_width(text, 12, expect);
}

#[test]
fn unary_operators() {
    let text = r##"let g = -x + y
let h = not x
"##;
    let expect = r##"let g = -x + y
let h = not x
"##;
    test_tmpl(text, expect);
}

#[test]
fn unary_operator_on_the_right() {
    let text = r##"let g = 1 + -2
let h = a && not b
"##;
    let expect = r##"let g = 1 + -2
let h = a && not b
"##;
    test_tmpl(text, expect);
}

#[test]
fn prefix_operators() {
    let text = r##"let g = !x ^ &y
"##;
    let expect = r##"let g = !x ^ &y
"##;
    test_tmpl(text, expect);
}

#[test]
fn field_access() {
    let text = r##"let g = r#field#other
"##;
    let expect = r##"let g = r#field#other
"##;
    test_tmpl(text, expect);
}

#[test]
fn expression_with_module() {
    let text = r##"let g = M.(f 1)
"##;
    let expect = r##"let g = M.(f 1)
"##;
    test_tmpl(text, expect);
}

#[test]
fn tuple_in_one_line() {
    let text = r##"let g = (1, 2, 3)
"##;
    let expect = r##"let g = (1, 2, 3)
"##;
    test_tmpl(text, expect);
}

#[test]
fn tuple_breaks_when_too_long() {
    let text = r##"let g = (aaaaaaaaaaaaa, bbbbbbbbbbbbb, ccccccccccccc)
"##;
    let expect = r##"let g = (
    aaaaaaaaaaaaa,
    bbbbbbbbbbbbb,
    ccccccccccccc
)
"##;
    test_width(text, 30, expect);
}

#[test]
fn unit() {
    let text = r##"let g = ()
"##;
    let expect = r##"let g = ()
"##;
    test_tmpl(text, expect);
}

#[test]
fn optional_arguments() {
    let text = r##"let g = f ?:x ?* y
"##;
    let expect = r##"let g = f ?:x ?* y
"##;
    test_tmpl(text, expect);
}

#[test]
fn command_application() {
    let text = r##"let g = command \foo
"##;
    let expect = r##"let g = command \foo
"##;
    test_tmpl(text, expect);
}

#[test]
fn constructors() {
    let text = r##"let a = Some 1
let b = Some(1)
let c = None
"##;
    let expect = r##"let a = Some 1
let b = Some(1)
let c = None
"##;
    test_tmpl(text, expect);
}

#[test]
fn record_in_one_line() {
    let text = r##"let g = (| a = 1; b = 2 |)
"##;
    let expect = r##"let g = (|a = 1; b = 2|)
"##;
    test_tmpl(text, expect);
}

#[test]
fn record_breaks_with_trailing_semicolon() {
    let text = r##"let g = (| aaaa = 1; bbbb = 2 |)
"##;
    let expect = r##"let g = (|
    aaaa = 1;
    bbbb = 2;
|)
"##;
    test_width(text, 16, expect);
}

#[test]
fn record_update() {
    let text = r##"let g = (| r with a = 1 |)
"##;
    let expect = r##"let g = (|r with a = 1|)
"##;
    test_tmpl(text, expect);
}

#[test]
fn empty_record_and_list() {
    let text = r##"let a = (| |)
let b = [ ]
"##;
    let expect = r##"let a = (||)
let b = []
"##;
    test_tmpl(text, expect);
}

#[test]
fn list_in_one_line() {
    let text = r##"let g = [1; 2; 3;]
"##;
    let expect = r##"let g = [1; 2; 3]
"##;
    test_tmpl(text, expect);
}

#[test]
fn list_breaks_with_trailing_semicolon() {
    let text = r##"let g = [aaaaaaaa; bbbbbbbb; cccccccc]
"##;
    let expect = r##"let g = [
    aaaaaaaa;
    bbbbbbbb;
    cccccccc;
]
"##;
    test_width(text, 20, expect);
}

#[test]
fn nested_list_breaks_only_the_outer() {
    let text = r##"let g = [[1; 2]; [3; 4]]
"##;
    let expect = r##"let g = [
    [1; 2];
    [3; 4];
]
"##;
    test_width(text, 18, expect);
}

#[test]
fn application_with_lists() {
    let text = r##"let g = f [1; 2] [3]
"##;
    let expect = r##"let g = f [1; 2] [3]
"##;
    test_tmpl(text, expect);
}

#[test]
fn application_of_parenthesized() {
    let text = r##"let g = (f x) (g y)
"##;
    let expect = r##"let g = (f x) (g y)
"##;
    test_tmpl(text, expect);
}

#[test]
fn application_breaks_arguments() {
    let text = r##"let y = f aaaaaaaa bbbbbbbb cccccccc
"##;
    let expect = r##"let y =
    f
        aaaaaaaa
        bbbbbbbb
        cccccccc
"##;
    test_width(text, 20, expect);
}

#[test]
fn application_hugs_the_last_list() {
    let text = r##"let y = vconcat [aaaaaaaa; bbbbbbbb; cccccccc]
"##;
    let expect = r##"let y = vconcat [
    aaaaaaaa;
    bbbbbbbb;
    cccccccc;
]
"##;
    test_width(text, 30, expect);
}

#[test]
fn application_breaks_parenthesized_arguments() {
    let text = r##"let g = foo (aaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbb) (cccccccccccccccc dddddddddddddddd)
"##;
    let expect = r##"let g =
    foo
        (aaaaaaaaaaaaaaaa
            bbbbbbbbbbbbbbbb)
        (cccccccccccccccc
            dddddddddddddddd)
"##;
    test_width(text, 40, expect);
}

#[test]
fn operators_break_before_the_operator() {
    let text = r##"let y = aaaaaaaa |> bbbbbbbb |> cccccccc
"##;
    let expect = r##"let y =
    aaaaaaaa
        |> bbbbbbbb
        |> cccccccc
"##;
    test_width(text, 20, expect);
}

#[test]
fn constants() {
    let text = r##"let a = 1.5pt + 2cm
let b = `str` ^ `a`
let c = (+) 1 2
"##;
    let expect = r##"let a = 1.5pt + 2cm
let b = `str` ^ `a`
let c = (+) 1 2
"##;
    test_tmpl(text, expect);
}

#[test]
fn long_string_literal_is_not_split() {
    let text = r##"let s = `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
"##;
    let expect = r##"let s =
    `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
"##;
    test_tmpl(text, expect);
}

#[test]
fn multiline_string_literal_is_not_reindented() {
    let text = r##"let s = `
  a
    b
`
let t = [`x
  y`; 1]
"##;
    let expect = r##"let s =
    `
  a
    b
`
let t = [
    `x
  y`;
    1;
]
"##;
    test_tmpl(text, expect);
}
