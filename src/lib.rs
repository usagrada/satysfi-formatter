#[cfg(test)]
mod tests;

use satysfi_parser::{grammar, Cst, CstText};

struct OptionData {
  row_length: usize,
  indent_space: usize,
}

// format 設定のオプション
static option: OptionData = OptionData {
  row_length: 40,
  indent_space: 4,
};

/// satysfi の文字列を渡すと format したものを返す
/// * `input` - satysfi のコード  
/// * `output` - format された文字列
pub fn format(input: &str) -> String {
  /*
  CstText {
    text: string,
    lines: Vec<usize>, // start
    cst: Cst,
  }
  */
  let csttext = CstText::parse(input, grammar::program).unwrap();
  let mut output = String::new();

  // dbg!(&csttext.cst);
  // dbg!(&csttext.pritty_cst_recursive(&csttext.cst));
  visualize_csttext_tree(&csttext);
  let depth = 0;
  for node in csttext.cst.inner.iter() {
    output += &to_string_cst_inner(input, node, depth);
  }

  // 末尾に改行がない場合、改行を挿入して終了
  if output.chars().nth_back(0) != Some('\n') {
    output += "\n";
  }

  output
}

// for debug
fn visualize_csttext_tree(csttext: &CstText) {
  for node in csttext.cst.inner.iter() {
    visualize_cst_tree(&csttext.text, node, 0);
  }
}

// for debug
fn visualize_cst_tree(text: &str, cst: &Cst, depth: usize) {
  // println!("{}{:?}: {{", " ".repeat(depth * 2), cst.rule);
  let start = cst.span.start;
  let end = std::cmp::min(start + 5, cst.span.end);
  let self_text = if end == cst.span.end {
    text.get(start..end).unwrap().trim().to_string()
  } else {
    text.get(start..end).unwrap().trim().to_string() + "..."
  };
  println!("{}* {:?}: {}", " ".repeat(depth * 2), cst.rule, self_text);
  // println!("{}└─ {:?}", cst.rule);
  for node in cst.inner.iter() {
    visualize_cst_tree(text, node, depth + 1);
  }
  // println!("{}}"," ".repeat(depth * 2));
}

fn to_string_cst_inner(text: &str, cst: &Cst, depth: usize) -> String {
  /*
  Cst {
    rule: Rule,
    span: Span { start: number, end: number },
    inner: [Cst] }
  */
  use satysfi_parser::Rule;
  let csts = cst.inner.clone();
  let sep = &match cst.rule {
    Rule::block_cmd | Rule::horizontal_single => " ".to_string(),
    Rule::vertical | Rule::horizontal_bullet_list => format!("\n{}", indent_space(depth)),
    Rule::record => format!(";\n{}", indent_space(depth)),
    _ => "".to_string(),
  };
  let output = csts.iter().fold(String::new(), |current, cst| {
    let s = to_string_cst(text, &cst, depth);
    if current.is_empty() {
      s
    } else if s.is_empty() {
      current
    } else {
      current + sep + &s
    }
  });

  output
}

// 中身をそのまま返すものは output をそのまま返す
fn to_string_cst(text: &str, cst: &Cst, depth: usize) -> String {
  if cst.rule == Rule::regular_text {
    println!(
      "{:?}, {:?}",
      cst.rule,
      text.get(cst.span.start..cst.span.end).unwrap()
    );
  }

  // インデントを制御するための変数
  let new_depth = match cst.rule {
    Rule::block_text | Rule::cmd_text_arg | Rule::record => depth + 1,
    _ => depth,
  };
  let start_indent = "\n".to_string() + &indent_space(new_depth);
  let end_indent = "\n".to_string() + &indent_space(depth);

  let output = to_string_cst_inner(text, cst, new_depth);
  let self_text = text.get(cst.span.start..cst.span.end).unwrap().to_string();

  use satysfi_parser::Rule;
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
        if output.len() > 0 {
          format!("'<{start_indent}{output}{end_indent}>")
        } else {
          format!("'<{output}>")
        }
      } else {
        if output.len() > 0 {
          format!("<{start_indent}{output}{end_indent}>")
        } else {
          format!("<{output}>")
        }
      }
    }
    Rule::horizontal_text => self_text,
    Rule::math_text => self_text,
    Rule::list => self_text,
    Rule::record => {
      // TODO: consider
      if cst.inner.len() > 1 {
        // 2 つ以上のときは改行
        format!("(|{start_indent}{output};{end_indent}|)")
      } else {
        // 1つだけの時は、改行しない
        format!("(|{output}|)")
      }
    }
    Rule::record_unit => self_text, // TODO
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
      // 括弧の種類を取得
      let start_arg = self_text.chars().nth(0).unwrap();
      let end_arg = self_text.chars().nth_back(0).unwrap();
      // 改行を含んでいたら、改行を入れる
      let include_kaigyou = output.find("\n") != None;
      match output.trim().len() {
        0 => format!("{start_arg}{end_arg}"),
        num if !include_kaigyou && num < option.row_length => format!("{start_arg} {output} {end_arg}"),
        _ => format!("{start_arg}{start_indent}{output}{end_indent}{end_arg}"),
      }
    }
    Rule::inline_cmd => {
      if self_text.chars().nth_back(0) == Some(';') {
        format!("{output};")
      } else {
        output
      }
    }
    Rule::inline_cmd_name => self_text,
    Rule::block_cmd => {
      format!("{output}")
    }

    Rule::block_cmd_name => self_text,
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
    Rule::horizontal_single => output,
    Rule::horizontal_list => output,                  // TODO
    Rule::horizontal_bullet_list => output,           // TODO
    Rule::horizontal_bullet => output,                // TODO
    Rule::horizontal_bullet_star => "* ".to_string(), // TODO
    Rule::regular_text => {
      let sep = format!("\n{}", indent_space(depth));
      let output = self_text
        .split("\n")
        .into_iter()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<String>>()
        .join(&sep);
      // remove space of start and end
      // self_text.trim().to_string()
      output
    }
    Rule::horizontal_escaped_char => output, // TODO
    Rule::inline_text_embedding => output,   // TODO

    // vertical
    Rule::vertical => output,             // TODO
    Rule::block_text_embedding => output, // TODO

    // TODO other things
    Rule::misc => " ".to_string(),
    Rule::program_saty => output,
    Rule::program_satyh => output,
    Rule::preamble => self_text,
    // TODO
    // _ => self_text,
    _ => "".to_string(),
  }
}

#[inline]
fn indent_space(depth: usize) -> String {
  let mut output = String::new();
  for _ in 0..option.indent_space * depth {
    output += " "
  }
  output
}
