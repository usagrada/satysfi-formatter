use satysfi_formatter::{format, visualize_csttext_tree};

// for debug called by main.rs
fn input() {
  let text = r#"document(|
      author = {author};
      show-title = false;
      show-toc = true;
      title = {title};
  |)'<
      
          +p {
              hello
              world${ax}
      }


  >"#;
  let output = format(text);
  println!("{output}");
}

fn test_unicode() {
  let text = r#"
    document(||)'<
  +section{ section }<
  +p {日本語}
  >>"#;
  let csttext =
    satysfi_parser::CstText::parse(text, satysfi_parser::grammar::program).expect("parse error");
  // let output = format(text);
  let expect = r#"document(||)'<
      +section { section } <
          +p {
              日本語
          }
      >
  >
  "#;
  visualize_csttext_tree(&csttext);
  // assert_eq!(output, expect);
}

fn main() {
  //   input();
  test_unicode();
}
