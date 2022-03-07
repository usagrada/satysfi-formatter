#[cfg(test)]
mod tests;

use satysfi_parser::{grammar, Cst, CstText};

static mut PROGRAM_TEXT: String = String::new();

struct OptionData {
  row_length: usize,
  indent_space: usize,
}

const option: OptionData = OptionData {
  row_length: 100,
  indent_space: 4,
};

// for debug
pub fn input() {
  let text = r#"@import: hello
@require: local
% comment
document(|title = hello|)'<+p{hello world}>"#;
  let output = format(text);
  println!("{}", output);
}

pub fn format(input: &str) -> String {
  /*
  CstText {
    text: string,
    lines: Vec<usize>, // start
    cst: Cst,
  }
  */
  let csttext = CstText::parse(input, grammar::program).unwrap();
  unsafe {
    PROGRAM_TEXT = input.to_string();
  }
  println!("csttext\n{:?}\n\n", csttext.cst);
  let mut output = String::new();
  let depth = 0;
  for node in csttext.cst.inner.iter() {
    output += &to_string_csts(node.inner.clone(), depth);
  }
  println!("end format\n");
  output
}

fn to_string_csts(csts: Vec<Cst>, depth: usize) -> String {
  /*
  Cst {
    rule: Rule,
    span: Span { start: number, end: number },
    inner: [Cst] }
  */
  let mut output = String::new();
  for cst in csts {
    output += &to_string_cst(&cst, depth);
  }

  output
}

// 中身をそのまま返すものは output をそのまま返す
fn to_string_cst(cst: &Cst, depth: usize) -> String {
  println!("{:?}, {:?}", cst.rule, unsafe {
    PROGRAM_TEXT.get(cst.span.start..cst.span.end).unwrap()
  });

  // インデントを制御するための変数
  let new_depth = match cst.rule {
    Rule::block_cmd => depth + 1,
    _ => depth,
  };

  use satysfi_parser::Rule;
  let output = to_string_csts(cst.inner.clone(), new_depth);
  let self_text = unsafe { PROGRAM_TEXT.get(cst.span.start..cst.span.end) }
    .unwrap()
    .to_string();

  match cst.rule {
    // header
    Rule::header_import => "@import: ".to_string() + &output + "\n",
    Rule::header_require => "@require: ".to_string() + &output + "\n",
    Rule::pkgname => self_text,

    // unary
    Rule::unary => output,
    Rule::unary_prefix => output,
    Rule::block_text => {
      if self_text.chars().nth(0) == Some('\'') {
        format!("'<\n{output}\n>")
      } else {
        format!("<\n{output}\n>")
      }
    }
    Rule::horizontal_text => self_text,
    Rule::math_text => self_text,
    Rule::list => self_text,
    Rule::record => self_text,
    Rule::record_unit => self_text,
    Rule::tuple => self_text,
    Rule::bin_operator => self_text,
    Rule::expr_with_mod => self_text,
    Rule::var => self_text,
    Rule::var_ptn => self_text,
    Rule::modvar => self_text,
    Rule::mod_cmd_name => self_text,
    Rule::module_name => self_text,
    Rule::variant_name => self_text,

    // command
    Rule::cmd_name_ptn => self_text,
    Rule::cmd_expr_arg => self_text,
    Rule::cmd_expr_option => self_text,
    Rule::cmd_text_arg => {
      let space = if output.len() > option.row_length {
        format!("\n{}", indent_space(depth))
      } else {
        " ".to_string()
      };
      format!(" {{{space}{output}{space}}}")
    }
    Rule::inline_cmd => self_text,
    Rule::inline_cmd_name => self_text,
    Rule::block_cmd => output,
    Rule::block_cmd_name => {
      format!("{}{}", indent_space(depth), self_text)
    }
    Rule::math_cmd => self_text,
    Rule::math_cmd_name => self_text,
    Rule::math_cmd_expr_arg => self_text,
    Rule::math_cmd_expr_option => self_text,

    // expr
    Rule::expr => output,
    Rule::match_expr => output,          // TODO
    Rule::match_arm => output,           // TODO
    Rule::match_guard => output,         // TODO
    Rule::bind_stmt => output,           // TODO
    Rule::ctrl_while => output,          // TODO
    Rule::ctrl_if => output,             // TODO
    Rule::lambda => output,              // TODO
    Rule::assignment => output,          // TODO
    Rule::dyadic_expr => output,         // TODO
    Rule::unary_operator_expr => output, // TODO
    Rule::unary_operator => output,      // TODO
    // application
    Rule::application => output,
    Rule::application_args_normal => output,
    Rule::application_args_optional => output,
    Rule::command_application => output,
    Rule::variant_constructor => output,

    // horizontal
    Rule::horizontal_single => output,      // TODO
    Rule::horizontal_list => output,        // TODO
    Rule::horizontal_bullet_list => output, // TODO
    Rule::horizontal_bullet => output,      // TODO
    Rule::horizontal_bullet_star => output, // TODO
    Rule::regular_text => self_text,
    Rule::horizontal_escaped_char => output, // TODO
    Rule::inline_text_embedding => output,   // TODO

    // vertical
    Rule::vertical => output,             // TODO
    Rule::block_text_embedding => output, // TODO

    // TODO other things
    Rule::misc => " ".to_string(),
    Rule::program_saty => " ".to_string(),
    Rule::program_satyh => " ".to_string(),
    Rule::preamble => " ".to_string(),
    // TODO
    // _ => self_text,
    _ => "".to_string(),
  }
}

fn indent_space(depth: usize) -> String {
  let mut output = String::new();
  for _ in 0..option.indent_space * depth {
    output += " "
  }
  output
}
