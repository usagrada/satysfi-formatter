use super::test_tmpl;

#[test]
fn test_const_string_sequently1() {
    let text = r#"document(||)'<
    +p{`ok` `no`
    }
>
"#;

    let expect = r#"document(||)'<
    +p { `ok` `no` }
>
"#;
    test_tmpl(text, expect)
}

#[test]
fn test_const_string_sequently2() {
    let text = r#"document(||)'<
    +p{`ok`
`no`
    }
>
"#;

    let expect = r#"document(||)'<
    +p {
        `ok`
        `no`
    }
>
"#;
    test_tmpl(text, expect)
}
