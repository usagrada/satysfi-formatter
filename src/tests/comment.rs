use super::test_tmpl;

#[test]
fn test_comment1() {
    let text = r#"@import: hello
  @require: local
  %comment
  
document(|title = {hello}|)'<+p{% comment
}>"#;

    let expect = r#"@import: hello
@require: local

% comment
document(|title = { hello }|)'<
    +p {
        % comment
    }
>
"#;
    test_tmpl(text, expect)
}

#[test]
fn test_comment2() {
    let text = r#"@import: hello
  @require: local
  %comment
  
document(|title = {hello}|)'<+p{hello% comment
}>"#;

    let expect = r#"@import: hello
@require: local

% comment
document(|title = { hello }|)'<
    +p {
        hello
        % comment
    }
>
"#;
    test_tmpl(text, expect)
}

#[test]
fn test_comment3() {
    let text = r#"@import: hello
  @require: local
  %comment
  
document(|title = {hello}|)'<+p{hello% comment
    \listing {
        * item1
        * item2
        * item3
    }
}>"#;

    let expect = r#"@import: hello
@require: local

% comment
document(|title = { hello }|)'<
    +p {
        hello
        % comment
        \listing {
            * item1
            * item2
            * item3
        }
    }
>
"#;
    test_tmpl(text, expect)
}

#[test]
fn test_comment4() {
    let text = r#"@import: hello
  @require: local
  %comment
  
document(|title = { hello }|)'<+p{% comment
    \listing {
        * item1
        * item2
        * item3
    }
}>"#;
    let expect = r#"@import: hello
@require: local

% comment
document(|title = { hello }|)'<
    +p {
        % comment
        \listing {
            * item1
            * item2
            * item3
        }
    }
>
"#;
    test_tmpl(text, expect)
}

#[test]
fn test_comment5() {
    let text = r#"@import: hello
  @require: local %comment
  
document(|title = {hello}|)'<+p% comment
{
    hello
}>"#;

    let expect = r#"@import: hello
@require: local %comment

document(|title = { hello }|)'<
    +p % comment
    { hello }
>
"#;
    test_tmpl(text, expect)
}


#[test]
fn test_comment6() {
    let text = r#"@import: hello
  @require: local %comment
  
document(|title = {hello}|)'<+p% comment
{
    hello
    \\%comment
}>"#;

    let expect = r#"@import: hello
@require: local %comment

document(|title = { hello }|)'<
    +p % comment
    {
        hello \\
        % comment
    }
>
"#;
    test_tmpl(text, expect)
}
