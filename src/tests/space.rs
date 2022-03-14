use super::test_tmpl;
#[test]
fn test_space1() {
    let text = r#"document(|title = { hello }|)'<+p% comment
{
    hello\bold{abc}def
}>"#;

    let expect = r#"document(|title = { hello }|)'<
    +p % comment
    { hello\bold { abc }def }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test_space2() {
    let text = r#"document(|title = {hello}|)'<+p% comment
{
    hello\bold{abc}def 
}>"#;

    let expect = r#"document(|title = { hello }|)'<
    +p % comment
    { hello\bold { abc }def }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test_space3() {
    let text = r#"document(|title = { hello }|)'<+p% comment
{
    hello\bold{abc}def 
}
+align [
  [${}; ${\( 1 + a \)   \( 1 + a^2 \)   \( 1 + a^{2^2} \)}];
  [${}; ${\paren{1 + a} \paren{1 + a^2} \paren{1 + a^{2^2}}}];
];
>"#;
    let expect = r#"document(|title = { hello }|)'<
    +p % comment
    { hello\bold { abc }def }
    +align [
        [
            ${};
            ${\( 1 + a \)   \( 1 + a^2 \)   \( 1 + a^{2^2} \)};
        ];
        [
            ${};
            ${\paren{1 + a} \paren{1 + a^2} \paren{1 + a^{2^2}}};
        ];
    ];
>
"#;
    test_tmpl(text, expect);
}
