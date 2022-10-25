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

impl Into<Token> for &str {
    fn into(self) -> Token {
        match self {
            "whitespace" => Token::whitespace,
            "cmd_name" => Token::cmd_name,
            "extras" => Token::extras,
            "word" => Token::word,
            "supertype" => Token::supertype,
            "conflicts" => Token::conflicts,
            "externals" => Token::externals,
            "literal_string" => Token::literal_string,
            "inline_token" => Token::inline_token,
            "source_file" => Token::source_file,
            "comment" => Token::comment,
            "program_saty" => Token::program_saty,
            "program_satyh" => Token::program_satyh,
            "headers" => Token::headers,
            "_header" => Token::_header,
            "header_require" => Token::header_require,
            "header_import" => Token::header_import,
            "header_stage" => Token::header_stage,
            "preamble" => Token::preamble,
            "_statement" => Token::_statement,
            "let_stmt" => Token::let_stmt,
            "_let_stmt_argument" => Token::_let_stmt_argument,
            "let_rec_stmt" => Token::let_rec_stmt,
            "let_rec_inner" => Token::let_rec_inner,
            "_let_rec_stmt_argument" => Token::_let_rec_stmt_argument,
            "let_rec_matcharm" => Token::let_rec_matcharm,
            "let_inline_stmt" => Token::let_inline_stmt,
            "let_block_stmt" => Token::let_block_stmt,
            "let_math_stmt" => Token::let_math_stmt,
            "let_mutable_stmt" => Token::let_mutable_stmt,
            "type_stmt" => Token::type_stmt,
            "type_inner" => Token::type_inner,
            "type_variant" => Token::type_variant,
            "open_stmt" => Token::open_stmt,
            "_arg" => Token::_arg,
            "module_stmt" => Token::module_stmt,
            "sig_stmt" => Token::sig_stmt,
            "struct_stmt" => Token::struct_stmt,
            "_sig_inner" => Token::_sig_inner,
            "sig_type_stmt" => Token::sig_type_stmt,
            "sig_val_stmt" => Token::sig_val_stmt,
            "sig_direct_stmt" => Token::sig_direct_stmt,
            "_type_expr" => Token::_type_expr,
            "type_fun" => Token::type_fun,
            "type_prod" => Token::type_prod,
            "type_inline_cmd" => Token::type_inline_cmd,
            "type_block_cmd" => Token::type_block_cmd,
            "type_math_cmd" => Token::type_math_cmd,
            "type_list" => Token::type_list,
            "type_record" => Token::type_record,
            "type_record_unit" => Token::type_record_unit,
            "type_application" => Token::type_application,
            "type_param" => Token::type_param,
            "type_name" => Token::type_name,
            "constraint" => Token::constraint,
            "pat_as" => Token::pat_as,
            "_pat_cons" => Token::_pat_cons,
            "_pattern" => Token::_pattern,
            "pat_variant" => Token::pat_variant,
            "pat_list" => Token::pat_list,
            "pat_tuple" => Token::pat_tuple,
            "_expr" => Token::_expr,
            "match_expr" => Token::match_expr,
            "match_arm" => Token::match_arm,
            "match_guard" => Token::match_guard,
            "bind_stmt" => Token::bind_stmt,
            "ctrl_while" => Token::ctrl_while,
            "ctrl_if" => Token::ctrl_if,
            "lambda" => Token::lambda,
            "assignment" => Token::assignment,
            "binary_expr" => Token::binary_expr,
            "binary_operator" => Token::binary_operator,
            "unary_operator_expr" => Token::unary_operator_expr,
            "unary_operator" => Token::unary_operator,
            "unary_prefix" => Token::unary_prefix,
            "application" => Token::application,
            "_application_args" => Token::_application_args,
            "_application_args_opt" => Token::_application_args_opt,
            "command_application" => Token::command_application,
            "variant_constructor" => Token::variant_constructor,
            "record_member" => Token::record_member,
            "_unary" => Token::_unary,
            "record" => Token::record,
            "_record_inner" => Token::_record_inner,
            "record_unit" => Token::record_unit,
            "list" => Token::list,
            "tuple" => Token::tuple,
            "expr_with_mod" => Token::expr_with_mod,
            "modvar" => Token::modvar,
            "_mod_cmd_name" => Token::_mod_cmd_name,
            "module_name" => Token::module_name,
            "variant_name" => Token::variant_name,
            "_literal" => Token::_literal,
            "identifier" => Token::identifier,
            "literal_unit" => Token::literal_unit,
            "literal_bool" => Token::literal_bool,
            "literal_length" => Token::literal_length,
            "literal_int" => Token::literal_int,
            "literal_float" => Token::literal_float,
            "inline_cmd" => Token::inline_cmd,
            "inline_cmd_name" => Token::inline_cmd_name,
            "block_cmd" => Token::block_cmd,
            "block_cmd_name" => Token::block_cmd_name,
            "cmd_expr_arg" => Token::cmd_expr_arg,
            "cmd_expr_option" => Token::cmd_expr_option,
            "cmd_text_arg" => Token::cmd_text_arg,
            "_cmd_expr_arg_inner" => Token::_cmd_expr_arg_inner,
            "math_cmd" => Token::math_cmd,
            "math_cmd_name" => Token::math_cmd_name,
            "math_cmd_expr_arg" => Token::math_cmd_expr_arg,
            "math_cmd_expr_option" => Token::math_cmd_expr_option,
            "_math_cmd_expr_arg_inner" => Token::_math_cmd_expr_arg_inner,
            "inline_text" => Token::inline_text,
            "inline_text_list" => Token::inline_text_list,
            "inline_text_bullet_list" => Token::inline_text_bullet_list,
            "horizontal" => Token::horizontal,
            "_horizontal_compound" => Token::_horizontal_compound,
            "inline_text_bullet_item" => Token::inline_text_bullet_item,
            "inline_text_bullet_star" => Token::inline_text_bullet_star,
            "inline_literal_escaped" => Token::inline_literal_escaped,
            "inline_text_embedding" => Token::inline_text_embedding,
            "block_text" => Token::block_text,
            "vertical" => Token::vertical,
            "block_text_embedding" => Token::block_text_embedding,
            "math_text" => Token::math_text,
            "math_list" => Token::math_list,
            "math" => Token::math,
            "math_token" => Token::math_token,
            "_math_sup" => Token::_math_sup,
            "_math_sub" => Token::_math_sub,
            "_math_group" => Token::_math_group,
            "math_unary" => Token::math_unary,
            "math_embedding" => Token::math_embedding,
            s => Token::other(s.to_string()),
        }
    }
}

pub fn token_to_string(token: Token) -> String {
    token.to_string()
}

pub const LIST_EXPR: [Token; 13] = [
    Token::match_expr,
    Token::bind_stmt,
    Token::ctrl_while,
    Token::ctrl_if,
    Token::lambda,
    Token::assignment,
    Token::binary_expr,
    Token::application,
    Token::unary_operator_expr,
    Token::command_application,
    Token::variant_constructor,
    Token::record_member,
    Token::_unary,
];

pub const LIST_UNARY: [Token; 18] = [
    Token::block_text,
    Token::inline_text,
    Token::inline_text_list,
    Token::inline_text_bullet_list,
    Token::math_text,
    Token::math_list,
    Token::record,
    Token::list,
    Token::tuple,
    // Token::binary_operator,
    // Token::_expr,
    Token::expr_with_mod,
    Token::modvar,
    // Token::_literal,
    Token::literal_unit,
    Token::literal_bool,
    Token::literal_length,
    Token::literal_int,
    Token::literal_string,
    Token::literal_float,
    Token::identifier,
];

pub const LIST_RECORD_INNER: [Token; 1] = [Token::record_unit];

pub const LIST_LITERAL: [Token; 6] = [
    Token::literal_unit,
    Token::literal_bool,
    Token::literal_length,
    Token::literal_int,
    Token::literal_string,
    Token::literal_float,
];

pub const LIST_TYPE_EXPR: [Token; 9] = [
    Token::type_fun,
    Token::type_prod,
    Token::type_inline_cmd,
    Token::type_block_cmd,
    Token::type_math_cmd,
    Token::type_application,
    Token::type_record,
    Token::type_param,
    Token::type_name,
];

#[test]
fn token_test() {
    let token = Token::word;
    assert_eq!(token.value(), "word");
}
