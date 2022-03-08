use satysfi_formatter::format;

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

fn main() {
  input();
}
