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
>"#;
    assert_eq!(output, expect);
}
