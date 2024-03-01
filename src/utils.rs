pub trait FormatUtility {
    fn add_word(&mut self, word: &str);
    fn add_newline(&mut self, indent_size: usize);
}
