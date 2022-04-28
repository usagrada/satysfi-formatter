use super::test_tmpl;

#[test]
fn test1() {
    let text = r#"@import: hello
  @require: local
  
document(|title = {hello}|)'<+p{hello world}>"#;
    let expect = r#"@import: hello
@require: local

document(|title = { hello }|)'<
    +p { hello world }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test2() {
    let text = r#"@import: hello
  @require: local
  
  
document(|title = {hello}|)'<+p{hello world}+p { hello world }>"#;

    let expect = r#"@import: hello
@require: local

document(|title = { hello }|)'<
    +p { hello world }
    +p { hello world }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test3() {
    let text = r#"@import: hello
@require: local


document(|title = {hello}|)'<+p{hello world}+p { \SATYSFI; }>"#;
    let expect = r#"@import: hello
@require: local

document(|title = { hello }|)'<
    +p { hello world }
    +p { \SATYSFI; }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test4() {
    let text = r#"@import: hello
@require: local

document(|title = {hello}|)'<+p{hello world}+p {\SATYSFI;format}>"#;

    let expect = r#"@import: hello
@require: local

document(|title = { hello }|)'<
    +p { hello world }
    +p { \SATYSFI;format }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test5() {
    let text = r#"@import: hello
@require: local

document(|title = {hello}|)'<+p{hello world}+p {format\SATYSFI;format}>"#;
    let expect = r#"@import: hello
@require: local

document(|title = { hello }|)'<
    +p { hello world }
    +p { format\SATYSFI;format }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test6() {
    let text = r#"

document(|title = {hello}|)'<+p{hello world}+p {${ax^2 + bx + c = 0}}>"#;
    let expect = r#"document(|title = { hello }|)'<
    +p { hello world }
    +p { ${a x^2 + bx + c = 0} }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test7() {
    let text = r#"
document(|title = {hello}; author = {author};|)'<>"#;
    let expect = r#"document(|
    title = { hello };
    author = { author };
|)'<>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test8() {
    let text = r#"@require: stdja
@require: itemize


document(|
    author = { author };
    show-title = false;
    show-toc = true;
    title = { title };
|)'<
    +section {section} <
        +p {
            
            \listing {
                * item1
                * item2
                * item3
            }
    }
    >
    
>"#;
    let expect = r#"@require: stdja
@require: itemize

document(|
    author = { author };
    show-title = false;
    show-toc = true;
    title = { title };
|)'<
    +section { section } <
        +p {
            \listing {
                * item1
                * item2
                * item3
            }
        }
    >
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test9() {
    let text = r#"
  
document(||)'<
+section{ section }<
+p {
        \listing{
            * item1
            * item2
            * item3
        }
}
>>"#;

    let expect = r#"document(||)'<
    +section { section } <
        +p {
            \listing {
                * item1
                * item2
                * item3
            }
        }
    >
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test10() {
    let text = r#"
  
document(||)'<
+section{ section }<
+p {
hello
        world
}
>>"#;
    let expect = r#"document(||)'<
    +section { section } <
        +p {
            hello
            world
        }
    >
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test11() {
    let text = r#"@import: hello
@require: local

document(|title = {hello}|)'<+p{hello world}+p {format\SATYSFI;
format}>"#;

    let expect = r#"@import: hello
@require: local

document(|title = { hello }|)'<
    +p { hello world }
    +p {
        format\SATYSFI;
        format
    }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test12() {
    // 改行が含まれていてもその前の末尾にスペースがある場合はスペースを優先
    let text = r#"@import: hello
@require: local

document(|title = {hello}|)'<+p{hello world}+p {format\SATYSFI; 
    format}>"#;

    let expect = r#"@import: hello
@require: local

document(|title = { hello }|)'<
    +p { hello world }
    +p { format\SATYSFI; format }
>
"#;
    test_tmpl(text, expect);
}
