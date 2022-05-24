use super::test_tmpl;

#[test]
fn test1() {
    let text = r#"
@require: stdja

document(||)'<
    +p{
        ${
\lim_{n\to\infty}a_n = a
        }
    }
>
"#;

    let expect = r#"@require: stdja

document(||)'<
    +p { ${\lim_{n \to \infty} a_n = a} }
>
"#;
    test_tmpl(text, expect)
}
