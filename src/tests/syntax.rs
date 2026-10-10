use super::test_tmpl;

#[test]
fn sequential_expr() {
    let text = "let f () = a <- 1 before b <- 2 before c\nin\ndocument(||)'<>";
    let expect = "let f () = a <- 1 before b <- 2 before c\nin\n\ndocument (||) '<>\n";
    test_tmpl(text, expect);
}

#[test]
fn path_expr() {
    let text = "let p = <[ (0pt, 0pt) -- (1pt, 1pt) -- cycle ]>\nin\ndocument(||)'<>";
    let expect = "let p = <[ (0pt, 0pt) -- (1pt, 1pt) -- cycle ]>\nin\n\ndocument (||) '<>\n";
    test_tmpl(text, expect);
}
