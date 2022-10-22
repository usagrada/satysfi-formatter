#[inline]
pub fn indent_space(depth: usize) -> String {
    let mut result = String::new();
    for _ in 0..depth {
        result.push_str(" ");
    }
    result
}
