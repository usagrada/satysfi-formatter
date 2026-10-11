//! 文 (`let` / `type` / `module` など) の整形
use super::{test_tmpl, test_width};

#[test]
fn let_with_arguments() {
    let text = r#"let f x y = x + y
"#;
    let expect = r#"let f x y = x + y
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_with_type_annotation() {
    let text = r#"let f : int -> int | x = x
"#;
    let expect = r#"let f : int -> int | x = x
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_rec_and() {
    let text = r#"let-rec f x = g x
and g y = y
"#;
    let expect = r#"let-rec f x = g x
and g y = y
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_rec_arms_without_body() {
    let text = r#"let-rec f
| 0 = 1
| n = n * f (n - 1)
"#;
    let expect = r#"let-rec f
    | 0 = 1
    | n = n * f (n - 1)
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_rec_arms_after_body() {
    let text = r#"let-rec f x = x
| y = y
"#;
    let expect = r#"let-rec f x = x
    | y = y
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_inline_with_context() {
    let text = r#"let-inline ctx \foo x y = {#x; #y;}
"#;
    let expect = r#"let-inline ctx \foo x y = { #x; #y; }
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_inline_without_context() {
    let text = r#"let-inline \foo x = {#x;}
"#;
    let expect = r#"let-inline \foo x = { #x; }
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_block_with_context() {
    let text = r#"let-block ctx +bar x = '<+p{#x;}>
"#;
    let expect = r#"let-block ctx +bar x = '<
    +p{ #x; }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_block_without_context() {
    let text = r#"let-block +bar x = x
"#;
    let expect = r#"let-block +bar x = x
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_math() {
    let text = r#"let-math \sq x = ${\sqrt{#x}}
"#;
    let expect = r#"let-math \sq x = ${\sqrt{#x}}
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_mutable() {
    let text = r#"let-mutable m <- 0
"#;
    let expect = r#"let-mutable m <- 0
"#;
    test_tmpl(text, expect);
}

#[test]
fn type_variants_in_one_line() {
    let text = r#"type t = A | B of int | C of int * string
"#;
    let expect = r#"type t = A | B of int | C of int * string
"#;
    test_tmpl(text, expect);
}

#[test]
fn type_variants_break_with_leading_bar() {
    let text = r#"type t = AAAAAAAAAAA | BBBBBBBBBBB of int | CCCCCCCCCCCC of string
"#;
    let expect = r#"type t =
    | AAAAAAAAAAA
    | BBBBBBBBBBB of int
    | CCCCCCCCCCCC of string
"#;
    test_width(text, 40, expect);
}

#[test]
fn type_with_parameter() {
    let text = r#"type 'a t = A of 'a | B
"#;
    let expect = r#"type 'a t = A of 'a | B
"#;
    test_tmpl(text, expect);
}

#[test]
fn type_synonym() {
    let text = r#"type t = int
"#;
    let expect = r#"type t = int
"#;
    test_tmpl(text, expect);
}

#[test]
fn type_and() {
    let text = r#"type t = A | B and u = C | D
"#;
    let expect = r#"type t = A | B
and u = C | D
"#;
    test_tmpl(text, expect);
}

#[test]
fn type_constraint() {
    let text = r#"type 'a t = A constraint 'a :: (| x : int |)
"#;
    let expect = r#"type 'a t = A
    constraint 'a :: (|x: int|)
"#;
    test_tmpl(text, expect);
}

#[test]
fn open() {
    let text = r#"open Foo
"#;
    let expect = r#"open Foo
"#;
    test_tmpl(text, expect);
}

#[test]
fn empty_module() {
    let text = r#"module M = struct end
"#;
    let expect = r#"module M = struct end
"#;
    test_tmpl(text, expect);
}

#[test]
fn empty_module_with_signature() {
    let text = r#"module M : sig end = struct end
"#;
    let expect = r#"module M : sig end = struct end
"#;
    test_tmpl(text, expect);
}

#[test]
fn module_with_signature_and_blank_lines() {
    let text = r#"module M : sig
 type t
 type 'a u
 val f : t -> u
 val (+) : t -> t -> t
 val +p : [inline-text] block-cmd
 direct \x : [int] inline-cmd
end = struct
 let f x = x


 let g x = x
 module N = struct let h = 1 end
 let i = 2
end
"#;
    let expect = r#"module M : sig
    type t
    type 'a u
    val f : t -> u
    val (+) : t -> t -> t
    val +p : [inline-text] block-cmd
    direct \x : [int] inline-cmd
end = struct
    let f x = x

    let g x = x

    module N = struct
        let h = 1
    end

    let i = 2
end
"#;
    test_tmpl(text, expect);
}

#[test]
fn blank_lines_are_kept_up_to_one() {
    let text = r#"let a = 1


let b = 2
let c = 3

let d = 4
"#;
    let expect = r#"let a = 1

let b = 2
let c = 3

let d = 4
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_in_chain() {
    let text = r#"let x =
  let y = 1 in
  let z = 2 in
  y + z
"#;
    let expect = r#"let x =
    let y = 1 in
    let z = 2 in
    y + z
"#;
    test_tmpl(text, expect);
}

#[test]
fn let_in_on_one_line() {
    let text = r#"let x = let y = 1 in y
"#;
    let expect = r#"let x =
    let y = 1 in
    y
"#;
    test_tmpl(text, expect);
}

#[test]
fn bind_open_mutable_and_rec() {
    let text = r#"let f x =
  open Foo in
  let-mutable a <- 1 in
  let-rec g y = y in
  a
"#;
    let expect = r#"let f x =
    open Foo in
    let-mutable a <- 1 in
    let-rec g y = y in
    a
"#;
    test_tmpl(text, expect);
}

#[test]
fn stage_and_headers() {
    let text = r#"@stage: 0
@require: a
let x = 1
"#;
    let expect = r#"@stage: 0

@require: a

let x = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn persistent_stage() {
    let text = r#"@stage: persistent
let x = 1
"#;
    let expect = r#"@stage: persistent

let x = 1
"#;
    test_tmpl(text, expect);
}

#[test]
fn blank_lines_between_headers_are_removed() {
    let text = r#"@require: a
@require: b


@import: c
document(||)'<>
"#;
    let expect = r#"@require: a
@require: b
@import: c

document(||)'<>
"#;
    test_tmpl(text, expect);
}

#[test]
fn preamble_and_body() {
    let text = r#"let x = 1
in
document(||)'<>
"#;
    let expect = r#"let x = 1
in

document(||)'<>
"#;
    test_tmpl(text, expect);
}

#[test]
fn blank_lines_at_end_are_removed() {
    let text = r#"document(||)'<>


"#;
    let expect = r#"document(||)'<>
"#;
    test_tmpl(text, expect);
}

#[test]
fn long_function_header_fills_arguments() {
    // 入るだけ引数を詰め、続きは本体より 1 段深く字下げする。パターンは分解しない
    let text = r#"let arrow-scheme strokef color lenL ((x1, y1) as pt1) ((x2, y2) as pt2) =
  Geom.atan2-point pt2 pt1
"#;
    let expect = r#"let arrow-scheme strokef color lenL ((x1, y1) as pt1)
        ((x2, y2) as pt2) =
    Geom.atan2-point pt2 pt1
"#;
    test_width(text, 60, expect);
}

#[test]
fn long_command_header_fills_arguments() {
    let text = r#"let-block ctx +theorem ?:wordopt ?:labelopt inner-block-text = inner
"#;
    let expect = r#"let-block ctx +theorem ?:wordopt ?:labelopt
        inner-block-text =
    inner
"#;
    test_width(text, 50, expect);
}
