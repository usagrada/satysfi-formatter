mod list;
pub use list::Token;

impl Token {
    pub fn value(self) -> String {
        self.to_string()
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[test]
fn token(){
    let token = Token::word;
    assert_eq!(token.value(), "word");
}