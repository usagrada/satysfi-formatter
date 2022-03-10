use satysfi_parser::{Cst, CstText, Rule};

// for debug
/// visualize
/// * `csttext:` - CstText::parse で取得したものを渡す
pub fn visualize_csttext_tree(csttext: &CstText) {
    visualize_cst_tree(&csttext, &csttext.cst, 0);
    for node in csttext.cst.inner.iter() {
        visualize_cst_tree(&csttext, node, 1);
    }
}

// for debug
fn visualize_cst_tree(csttext: &CstText, cst: &Cst, depth: usize) {
    let max_len = std::cmp::min(cst.span.end - cst.span.start, 10);
    let self_text = csttext
        .get_text_from_span(cst.span)
        .chars()
        .take(max_len)
        .collect::<String>()
        .replace("\n", ""); // 改行を削除
                            // overlide for 省略の表示
    let self_text = if cst.span.end - cst.span.start <= max_len {
        self_text
    } else {
        if cst.rule == Rule::regular_text {
            csttext.get_text_from_span(cst.span).to_string()
        } else {
            format!("{}...", self_text)
        }
    };
    println!("{}* {:?}: {}", " ".repeat(depth * 2), cst.rule, self_text);
    // println!("{}└─ {:?}", cst.rule);
    for node in cst.inner.iter() {
        visualize_cst_tree(csttext, node, depth + 1);
    }
}
