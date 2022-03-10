use crate::format;

#[test]
fn test1() {
    let text1 = r#"@import: hello
  @require: local
  
document(|title = {hello}|)'<+p{hello world}>"#;
    let output = format(text1);
    let expect = r#"@import: hello
@require: local

document(|title = {hello}|)'<
    +p { hello world }
>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test2() {
    let text2 = r#"@import: hello
  @require: local
  
  
document(|title = {hello}|)'<+p{hello world}+p { hello world }>"#;
    let output = format(text2);
    let expect = r#"@import: hello
@require: local

document(|title = {hello}|)'<
    +p { hello world }
    +p { hello world }
>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test3() {
    let text = r#"@import: hello
@require: local


document(|title = {hello}|)'<+p{hello world}+p { \SATYSFI; }>"#;
    let output = format(text);
    let expect = r#"@import: hello
@require: local

document(|title = {hello}|)'<
    +p { hello world }
    +p { \SATYSFI; }
>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test4() {
    let text = r#"@import: hello
@require: local

document(|title = {hello}|)'<+p{hello world}+p {\SATYSFI;format}>"#;
    let output = format(text);
    let expect = r#"@import: hello
@require: local

document(|title = {hello}|)'<
    +p { hello world }
    +p { \SATYSFI; format }
>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test5() {
    let text = r#"@import: hello
@require: local

document(|title = {hello}|)'<+p{hello world}+p {format\SATYSFI;format}>"#;
    let output = format(text);
    let expect = r#"@import: hello
@require: local

document(|title = {hello}|)'<
    +p { hello world }
    +p { format \SATYSFI; format }
>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test6() {
    let text = r#"

document(|title = {hello}|)'<+p{hello world}+p {${ax^2 + bx + c = 0}}>"#;
    let output = format(text);
    let expect = r#"
document(|title = {hello}|)'<
    +p { hello world }
    +p { ${ax^2 + bx + c = 0} }
>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test7() {
    let text = r#"
document(|title = {hello}; author = {author};|)'<>"#;
    let output = format(text);
    let expect = r#"
document(|
    title = {hello};
    author = {author};
|)'<>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test8() {
    let text = r#"@require: stdja
@require: itemize


document(|
    author = {author};
    show-title = false;
    show-toc = true;
    title = {title};
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
    let output = format(text);
    let expect = r#"@require: stdja
@require: itemize

document(|
    author = {author};
    show-title = false;
    show-toc = true;
    title = {title};
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
    assert_eq!(output, expect);
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
    let output = format(text);
    let expect = r#"
document(||)'<
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
    assert_eq!(output, expect);
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
    let output = format(text);
    let expect = r#"
document(||)'<
    +section { section } <
        +p {
            hello
            world
        }
    >
>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test_unicode() {
    let text = r#"
  
document(||)'<
+section{ section }<
+p {日本語}
>>"#;
    let output = format(text);
    let expect = r#"
document(||)'<
    +section { section } <
        +p { 日本語 }
    >
>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test_let_block2() {
    let text = r#"
@require: stdja

let-block ctx +newpage = clear-page
in

document(||)'<
    +newpage;
    
    +p{hello
    }
>
"#;
    let output = format(text);
    let expect = r#"@require: stdja

let-block ctx +newpage = clear-page
in

document(||)'<
    +newpage;
    +p { hello }
>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test_let_block3() {
    let text = r#"
@require: stdja

let-block ctx +newpage = clear-page
let-block ctx   +newcmd arg = '<+cmd{arg}>
in

document(||)'<
    +newpage;
    
    +p{hello
    }
>
"#;
    let output = format(text);
    let expect = r#"@require: stdja

let-block ctx +newpage = clear-page
let-block ctx +newcmd arg = '<
    +cmd { arg }
>
in

document(||)'<
    +newpage;
    +p { hello }
>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test_comment1() {
    let text1 = r#"@import: hello
  @require: local
  %comment
  
document(|title = {hello}|)'<+p{% comment
}>"#;
    let output = format(text1);
    let expect = r#"@import: hello
@require: local
% comment

document(|title = {hello}|)'<
    +p {
        % comment
    }
>
"#;
    assert_eq!(output, expect);
}

#[test]
fn test_comment2() {
    let text2 = r#"@import: hello
  @require: local
  %comment
  
document(|title = {hello}|)'<+p{hello% comment
}>"#;
    let output = format(text2);
    let expect = r#"@import: hello
@require: local
% comment

document(|title = {hello}|)'<
    +p {
        hello
        % comment
    }
>
"#;
    assert_eq!(output, expect);
}



#[test]
fn test_comment3() {
    let text3 = r#"@import: hello
  @require: local
  %comment
  
document(|title = {hello}|)'<+p{hello% comment
    \listing {
        * item1
        * item2
        * item3
    }
}>"#;
    let output = format(text3);
    let expect = r#"@import: hello
@require: local
% comment

document(|title = {hello}|)'<
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
    assert_eq!(output, expect);
}

#[test]
fn test_comment4() {
    let text3 = r#"@import: hello
  @require: local
  %comment
  
document(|title = {hello}|)'<+p{% comment
    \listing {
        * item1
        * item2
        * item3
    }
}>"#;
    let output = format(text3);
    let expect = r#"@import: hello
@require: local
% comment

document(|title = {hello}|)'<
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
    assert_eq!(output, expect);
}

#[test]
fn test_comment5() {
    let text3 = r#"@import: hello
  @require: local
  %comment
  
document(|title = {hello}|)'<+p% comment
{
    hello
}>"#;
    let output = format(text3);
    let expect = r#"@import: hello
@require: local
% comment

document(|title = {hello}|)'<
    +p % comment
    { hello }
>
"#;
    assert_eq!(output, expect);
}

