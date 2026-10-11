use super::test_tmpl;

#[test]
fn sequential_expr() {
    let text = r#"let f () = a <- 1 before b <- 2 before c
in
document(||)'<>"#;
    let expect = r#"let f () = a <- 1 before b <- 2 before c
in

document (||) '<>
"#;
    test_tmpl(text, expect);
}

#[test]
fn path_expr() {
    let text = r#"let p = <[ (0pt, 0pt) -- (1pt, 1pt) -- cycle ]>
in
document(||)'<>"#;
    let expect = r#"let p = <[ (0pt, 0pt) -- (1pt, 1pt) -- cycle ]>
in

document (||) '<>
"#;
    test_tmpl(text, expect);
}
