use crate::format;

#[test]
pub fn test1() {
  let text = r#"@import: hello
@require: local
% comment
document(|title = hello|)'<+p{hello world}>"#;
  let output = format(text);
  let expect = r#"@import: hello
@require: local
document(|title = hello|)'<
    +p { hello world }
>
"#;
  assert_eq!(output, expect);
}

#[test]
pub fn test2() {
  let text = r#"@import: hello
@require: local
% comment
document(|title = hello|)'<+p{hello world}+p { hello world }>"#;
  let output = format(text);
  let expect = r#"@import: hello
@require: local
document(|title = hello|)'<
    +p { hello world }
    +p { hello world }
>
"#;
  assert_eq!(output, expect);
}

#[test]
pub fn test3() {
  let text = r#"@import: hello
@require: local
% comment
document(|title = hello|)'<+p{hello world}+p { \SATYSFI; }>"#;
  let output = format(text);
  let expect = r#"@import: hello
@require: local
document(|title = hello|)'<
    +p { hello world }
    +p { \SATYSFI; }
>
"#;
  assert_eq!(output, expect);
}
