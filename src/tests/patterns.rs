//! パターンの整形
use super::test_tmpl;

#[test]
fn tuple_and_list() {
    let text = r#"let f (a, b) [c; d] = a
"#;
    let expect = r#"let f (a, b) [c; d] = a
"#;
    test_tmpl(text, expect);
}

#[test]
fn cons_and_as() {
    let text = r#"let f (x :: xs) (Some y as z) = y
"#;
    let expect = r#"let f (x :: xs) (Some y as z) = y
"#;
    test_tmpl(text, expect);
}

#[test]
fn constructor_with_parentheses_is_kept() {
    let text = r#"let f (Some(x)) = x
"#;
    let expect = r#"let f (Some(x)) = x
"#;
    test_tmpl(text, expect);
}

#[test]
fn constructor_with_space_is_kept() {
    let text = r#"let f (Some x) = x
"#;
    let expect = r#"let f (Some x) = x
"#;
    test_tmpl(text, expect);
}

#[test]
fn operator_and_wildcard() {
    let text = r#"let f (+) _ = 0
"#;
    let expect = r#"let f (+) _ = 0
"#;
    test_tmpl(text, expect);
}

#[test]
fn nested() {
    let text = r#"let f ((a, b), [c]) = a
"#;
    let expect = r#"let f ((a, b), [c]) = a
"#;
    test_tmpl(text, expect);
}

#[test]
fn match_patterns() {
    let text = r#"let g = match x with
| A -> 1
| B y when y > 0 -> 2
| _ -> 3
"#;
    let expect = r#"let g =
    match x with
    | A -> 1
    | B y when y > 0 -> 2
    | _ -> 3
"#;
    test_tmpl(text, expect);
}
