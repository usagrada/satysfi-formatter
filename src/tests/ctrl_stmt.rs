use super::test_tmpl;

#[test]
fn if_stmt() {
    let text = r#"@import: hello
  @require: local
  let-mutable x <- 0
  let () = if (1 < 2) then x <- 1 else x <- 2
in

document(|title = {hello}|)'<
>"#;

    let expect = r#"@import: hello
@require: local

let-mutable x <- 0
let () = if (1 < 2) then x <- 1 else x <- 2
in

document(|title = { hello }|)'<>
"#;

    test_tmpl(text, expect)
}

#[test]
fn while_stmt() {
    let text = r#"@import: hello
  @require: local
  let-mutable x <- 0
let () = while (!x < 2) do x <- !x + 1
let-inline \inline =
  let text = embed-string (arabic !x) in
  {#text;}
in

document(|title = {hello}|)'<
+p {
  \inline;
}
>"#;

    let expect = r#"@import: hello
@require: local

let-mutable x <- 0
let () = while (!x < 2) do x <- !x + 1
let-inline \inline =
    let text = embed-string (arabic !x) in { #text; }
in

document(|title = { hello }|)'<
    +p { \inline; }
>
"#;

    test_tmpl(text, expect)
}
