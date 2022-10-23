use tree_sitter::{Node, Tree};

pub struct App {
    state: usize,
}

pub fn visualize_csttext_tree(text: &str, tree: &Tree) {
    let app = App { state: 1 };
    println!("{}", app.state);
    println!("visualize");
    let root = tree.root_node();
    visualize_cst_tree(text, tree, root, 0);
    println!("\n============================================\n");
}

// for debug
fn visualize_cst_tree(text: &str, tree: &Tree, node: Node, depth: usize) {
    let range = node.byte_range();
    let mut output = text[range.start..range.end].trim();
    if output.contains("\n") {
        // output = &output[..output.find("\n").unwrap()];
        println!(
            "{}* {:?}: {}……",
            " ".repeat(depth * 2),
            node.kind(),
            // output,
            output.trim()
        );
    } else {
        println!(
            "{}* {:?}: {}",
            " ".repeat(depth * 2),
            node.kind(),
            output.trim(),
        );
    }

    for child in node.children(&mut node.walk()) {
        visualize_cst_tree(text, tree, child, depth + 1);
    }
}
