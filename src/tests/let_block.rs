use super::test_tmpl;

#[test]
fn test_let_block1() {
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

    let expect = r#"@require: stdja

let-block ctx +newpage = clear-page
in

document(||)'<
    +newpage;
    +p { hello }
>
"#;
    test_tmpl(text, expect)
}

#[test]
fn test_let_block2() {
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
    test_tmpl(text, expect);
}

#[test]
fn test_let_block3() {
    let text = r#"
@require: stdja
let x = '< +hoge; >
let a = '<
  #x;
>
in

document(||)'<
    +newpage;
    
    +p{hello
    }
>
"#;

    let expect = r#"@require: stdja

let x = '<
    +hoge;
>
let a = '<
    #x;
>
in

document(||)'<
    +newpage;
    +p { hello }
>
"#;
    test_tmpl(text, expect);
}

#[test]
fn test_let_block4() {
    let text = r#"
@require: stdja
let a = Some     
x#a   
in

document(||)'<
    +newpage;
    
    +p{hello
    }
>
"#;

    let expect = r#"@require: stdja

let a = Some x#a
in

document(||)'<
    +newpage;
    +p { hello }
>
"#;
    test_tmpl(text, expect);
}