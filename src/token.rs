mod list;
pub use {list::token_to_string, list::Token};

pub const LIST_EXPR: &[Token] = &[
    Token::expr_parened,
    Token::expr_constructor,
    Token::expr_application,
    Token::expr_var_path,
    Token::expr_lambda,
    Token::expr_bind,
    Token::expr_open,
    Token::expr_match,
    Token::expr_if,
    Token::expr_assignment,
    Token::expr_binary_operation,
    Token::expr_binary_operator,
    Token::expr_unary_operation,
    Token::inline_text,
    Token::block_text,
    Token::math_text,
    Token::expr_record,
    Token::expr_list,
    Token::expr_tuple,
    Token::expr_record_member,
    Token::expr_command,
    Token::_literal,
];

pub const LIST_RECORD_INNER: &[Token] = &[Token::record_unit];

pub const LIST_LITERAL: &[Token] = &[
    Token::literal_unit,
    Token::literal_bool,
    Token::literal_int,
    Token::literal_string,
    Token::literal_float,
    Token::literal_length,
];

pub const LIST_TYPE_EXPR: &[Token] = &[
    Token::type_inline_cmd,
    Token::type_block_cmd,
    Token::type_math_cmd,
    Token::type_application,
    Token::type_record,
    // Token::type_param,
    Token::type_name,
];
