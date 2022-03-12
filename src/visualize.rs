use satysfi_parser::{Cst, CstText, Rule};

// for debug
/// visualize
/// * `csttext:` - CstText::parse で取得したものを渡す
pub fn visualize_csttext_tree(csttext: &CstText) {
    println!("visualize");
    visualize_cst_tree(&csttext, &csttext.cst, 0);
}

// for debug
fn visualize_cst_tree(csttext: &CstText, cst: &Cst, depth: usize) {
    let max_len = std::cmp::min(cst.span.end - cst.span.start, 15);
    let self_text = csttext
        .get_text_from_span(cst.span)
        .chars();
    let self_text = if cst.span.end - cst.span.start <= max_len {
        self_text.take(max_len).collect::<String>()
        // .replace("\n", "") // 改行を削除
    } else {
        if cst.rule == Rule::regular_text {
            // 例外的に全部表示
            self_text.collect::<String>()
        } else {
            let start_text = self_text.clone().take(10).collect::<String>().replace("\n", ""); // 改行を削除
            let end_index = std::cmp::max(max_len, cst.span.end - cst.span.start - 5);
            let end_text = self_text.skip(end_index).take(cst.span.end - end_index).collect::<String>();
            format!("{}...{}", start_text, end_text)
        }
    };
    println!("{}* {:?}: start: {}, {}", " ".repeat(depth * 2), cst.rule, cst.span.start, self_text);
    // println!("{}└─ {:?}", cst.rule);
    for node in cst.inner.iter() {
        visualize_cst_tree(csttext, node, depth + 1);
    }
}
