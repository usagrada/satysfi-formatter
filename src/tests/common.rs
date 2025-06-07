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
    +p { ${ax^2 + bx + c = 0} }
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

#[test]
fn test13(){
    let text = r#"@require: local

document(|title = {}|)'<
    +fig-center(vconcat [
        gap 10pt;
        textbox { hello } 
            |> glass-box ?:(align-center, align-center) 100pt 100pt 
            |> bgcolor (Color.gray 0.8);
        gap 10pt;
    ]);
>"#;

    let expect = r#"@require: local

document(|title = {}|)'<
    +fig-center (vconcat [
        gap 10pt;
        textbox { hello }
            |> glass-box ?:(align-center, align-center) 100pt 100pt
            |> bgcolor (Color.gray 0.8);
        gap 10pt;
    ]);
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test14() {
    let text = r#"let f x = z"#;
    let expect = r#"let f x = z
"#;
    test_tmpl(text, expect);
}

#[test]
fn test15() {
    let text = r#"let f x =
let y = x in
let z = y in
 z"#;
    let expect = r#"let f x =
    let y = x in
    let z = y in
    z
"#;
    test_tmpl(text, expect);
}

#[test]
fn test16() {
    let text = r#"let f x =
let y = x in
let z = y in
 z

let a = 1
let b = 2
"#;
    let expect = r#"let f x =
    let y = x in
    let z = y in
    z
"#;
    test_tmpl(text, expect);
}
