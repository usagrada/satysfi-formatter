//! 型の整形 (`sig` の中の `val` で確認する)
use super::{test_tmpl, test_width};

#[test]
fn arrow_in_one_line() {
    let text = r#"module M : sig
 val f : int ?-> string -> bool
end = struct end
"#;
    let expect = r#"module M : sig
    val f : int ?-> string -> bool
end = struct end
"#;
    test_tmpl(text, expect);
}

#[test]
fn arrow_breaks_when_too_long() {
    let text = r#"module M : sig
 val f : aaaaaaaaaaaa -> bbbbbbbbbbbb -> cccccccccccc -> dddddddddddd
end = struct end
"#;
    let expect = r#"module M : sig
    val f : aaaaaaaaaaaa ->
        bbbbbbbbbbbb ->
        cccccccccccc ->
        dddddddddddd
end = struct end
"#;
    test_width(text, 40, expect);
}

#[test]
fn parenthesized_arrow_and_product() {
    let text = r#"module M : sig
 val g : (int -> int) -> int
 val h : int * string -> (int * int) option
end = struct end
"#;
    let expect = r#"module M : sig
    val g : (int -> int) -> int
    val h : int * string -> (int * int) option
end = struct end
"#;
    test_tmpl(text, expect);
}

#[test]
fn type_application() {
    let text = r#"module M : sig
 val i : 'a list -> 'a option
 val k : Foo.t -> Bar.u list
end = struct end
"#;
    let expect = r#"module M : sig
    val i : 'a list -> 'a option
    val k : Foo.t -> Bar.u list
end = struct end
"#;
    test_tmpl(text, expect);
}

#[test]
fn record_type() {
    let text = r#"module M : sig
 val j : (| a : int; b : string |) -> int
end = struct end
"#;
    let expect = r#"module M : sig
    val j : (|a: int; b: string|) -> int
end = struct end
"#;
    test_tmpl(text, expect);
}

#[test]
fn command_types() {
    let text = r#"module M : sig
 val f : [int?; string] inline-cmd
 val g : [] block-cmd
 val h : [int; (| x : int |)] math-cmd
end = struct end
"#;
    let expect = r#"module M : sig
    val f : [int?; string] inline-cmd
    val g : [] block-cmd
    val h : [int; (|x: int|)] math-cmd
end = struct end
"#;
    test_tmpl(text, expect);
}

#[test]
fn signature_items() {
    let text = r#"module M : sig
 type t
 type 'a u
 val (+++) : int -> int -> int
 direct \tabular : [inline-text list; (| l : int |)] inline-cmd
end = struct end
"#;
    let expect = r#"module M : sig
    type t
    type 'a u
    val (+++) : int -> int -> int
    direct \tabular : [inline-text list; (|l: int|)] inline-cmd
end = struct end
"#;
    test_tmpl(text, expect);
}
