//! 数式の整形
use super::test_tmpl;

#[test]
fn empty() {
    let text = r##"document(||)'<+p{${}}>
"##;
    let expect = r##"document (||) '<
    +p { ${} }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn spaces_are_kept_as_written() {
    let text = r##"document(||)'<+p{${a+b}}>
"##;
    let expect = r##"document (||) '<
    +p { ${a+b} }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn superscript_and_subscript() {
    let text = r##"document(||)'<+p{${a^2 b_i c_i^2 d^{x y}_{z w}}}>
"##;
    let expect = r##"document (||) '<
    +p { ${a^2 b_i c_i^2 d^{x y}_{z w}} }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn commands_with_arguments() {
    let text = r##"document(||)'<+p{${\frac{a}{b} \sqrt{x}}}>
"##;
    let expect = r##"document (||) '<
    +p { ${\frac{a}{b} \sqrt{x}} }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn math_list() {
    let text = r##"document(||)'<+p{${| a | b + c |}}>
"##;
    let expect = r##"document (||) '<
    +p { ${| a | b + c |} }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn multiline_math_breaks_braces() {
    let text = r##"document(||)'<+p{${a
  b
  c}}>
"##;
    let expect = r##"document (||) '<
    +p {
        ${
            a
            b
            c
        }
    }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn prime() {
    let text = r##"document(||)'<+p{${a'^2}}>
"##;
    let expect = r##"document (||) '<
    +p { ${a'^2} }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn embedding_in_math() {
    let text = r##"document(||)'<+p{${#x + \y{#z}}}>
"##;
    let expect = r##"document (||) '<
    +p { ${#x + \y{#z}} }
>
"##;
    test_tmpl(text, expect);
}

#[test]
fn all_kinds_of_command_arguments() {
    let text = r##"document(||)'<+p{${\f!{t}!<+p{x}>!(1)![1;2]!(|a=1|) \g?:{x}{y}}}>
"##;
    let expect = r##"document (||) '<
    +p {
        ${\f!{ t }!<
            +p { x }
        >!(1)![1; 2]!(|a = 1|) \g?:{x}{y}}
    }
>
"##;
    test_tmpl(text, expect);
}
