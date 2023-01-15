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
            "literal_string" => Token::literal_string,
            "inline_token" => Token::inline_token,
            "whitespace" => Token::whitespace,
            "cmd_name" => Token::cmd_name,
            "type_var" => Token::type_var,
            "var_name" => Token::var_name,
            "type_name" => Token::type_name,
            "variant_name" => Token::variant_name,
            "module_name" => Token::module_name,
            "label_name" => Token::label_name,
            "row_var" => Token::row_var,
            "extras" => Token::extras,
            "supertype" => Token::supertype,
            "conflicts" => Token::conflicts,
            "externals" => Token::externals,
            "source_file" => Token::source_file,
            "program_satyh" => Token::program_satyh,
            "program_saty" => Token::program_saty,
            "comment" => Token::comment,
            "headers" => Token::headers,
            "_header" => Token::_header,
            "header_stage" => Token::header_stage,
            "pkgname" => Token::pkgname,
            "_module" => Token::_module,
            "module_parened" => Token::module_parened,
            "module_path" => Token::module_path,
            "module_functor_abstraction" => Token::module_functor_abstraction,
            "module_functor_application" => Token::module_functor_application,
            "module_structure" => Token::module_structure,
            "module_coerction" => Token::module_coerction,
            "_binding" => Token::_binding,
            "bind_val" => Token::bind_val,
            "bind_type" => Token::bind_type,
            "bind_module" => Token::bind_module,
            "bind_signature" => Token::bind_signature,
            "bind_include" => Token::bind_include,
            "_bind_val" => Token::_bind_val,
            "_bind_val_single" => Token::_bind_val_single,
            "bind_val_variable" => Token::bind_val_variable,
            "bind_val_math_cmd" => Token::bind_val_math_cmd,
            "bind_val_inline_cmd" => Token::bind_val_inline_cmd,
            "bind_val_block_cmd" => Token::bind_val_block_cmd,
            "bind_val_mutable" => Token::bind_val_mutable,
            "bind_val_parameter" => Token::bind_val_parameter,
            "_bind_type" => Token::_bind_type,
            "bind_type_single" => Token::bind_type_single,
            "constructor_branch" => Token::constructor_branch,
            "opt_parameter" => Token::opt_parameter,
            "parameter" => Token::parameter,
            "_signature" => Token::_signature,
            "signature_parened" => Token::signature_parened,
            "signature_functor" => Token::signature_functor,
            "signature_with_bind_type" => Token::signature_with_bind_type,
            "signature_path" => Token::signature_path,
            "signature_structure" => Token::signature_structure,
            "_declaration" => Token::_declaration,
            "declaration_val" => Token::declaration_val,
            "_declaration_val_name" => Token::_declaration_val_name,
            "declaration_type_kind" => Token::declaration_type_kind,
            "declaration_type" => Token::declaration_type,
            "declaration_module" => Token::declaration_module,
            "declaration_signature" => Token::declaration_signature,
            "declaration_include" => Token::declaration_include,
            "type_kind" => Token::type_kind,
            "base_kind" => Token::base_kind,
            "row_kind" => Token::row_kind,
            "_type" => Token::_type,
            "type_path" => Token::type_path,
            "type_application" => Token::type_application,
            "type_function" => Token::type_function,
            "type_product" => Token::type_product,
            "type_parened" => Token::type_parened,
            "type_record" => Token::type_record,
            "type_math_cmd" => Token::type_math_cmd,
            "type_inline_cmd" => Token::type_inline_cmd,
            "type_block_cmd" => Token::type_block_cmd,
            "cmd_parameter_types" => Token::cmd_parameter_types,
            "cmd_parameter_type" => Token::cmd_parameter_type,
            "type_opts" => Token::type_opts,
            "type_opts_closed" => Token::type_opts_closed,
            "quant" => Token::quant,
            "_expr" => Token::_expr,
            "expr_parened" => Token::expr_parened,
            "expr_constructor" => Token::expr_constructor,
            "variant_path" => Token::variant_path,
            "expr_application" => Token::expr_application,
            "_expr_application_function" => Token::_expr_application_function,
            "expr_var_path" => Token::expr_var_path,
            "expr_lambda" => Token::expr_lambda,
            "expr_bind" => Token::expr_bind,
            "bind_val_pattern" => Token::bind_val_pattern,
            "expr_open" => Token::expr_open,
            "expr_match" => Token::expr_match,
            "match_arm" => Token::match_arm,
            "match_guard" => Token::match_guard,
            "expr_if" => Token::expr_if,
            "expr_binary_operation" => Token::expr_binary_operation,
            "expr_binary_operator" => Token::expr_binary_operator,
            "expr_unary_operation" => Token::expr_unary_operation,
            "expr_assignment" => Token::expr_assignment,
            "_literal" => Token::_literal,
            "expr_record" => Token::expr_record,
            "_record_inner" => Token::_record_inner,
            "record_unit" => Token::record_unit,
            "expr_list" => Token::expr_list,
            "expr_tuple" => Token::expr_tuple,
            "binary_operator" => Token::binary_operator,
            "_binary_expr" => Token::_binary_expr,
            "unary_operator" => Token::unary_operator,
            "expr_opts" => Token::expr_opts,
            "_expr_opts" => Token::_expr_opts,
            "variant" => Token::variant,
            "expr_record_member" => Token::expr_record_member,
            "expr_command" => Token::expr_command,
            "_matchable_const" => Token::_matchable_const,
            "_non_matchable_const" => Token::_non_matchable_const,
            "pattern_as" => Token::pattern_as,
            "_pattern_cons" => Token::_pattern_cons,
            "_pattern" => Token::_pattern,
            "_non_var_pattern" => Token::_non_var_pattern,
            "pattern_parened" => Token::pattern_parened,
            "pattern_operator" => Token::pattern_operator,
            "pattern_ignore" => Token::pattern_ignore,
            "pattern_variant" => Token::pattern_variant,
            "pattern_tuple" => Token::pattern_tuple,
            "pattern_const" => Token::pattern_const,
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
            "inline_cmd" => Token::inline_cmd,
            "inline_cmd_name" => Token::inline_cmd_name,
            "_mod_cmd_name" => Token::_mod_cmd_name,
            "block_cmd" => Token::block_cmd,
            "block_cmd_name" => Token::block_cmd_name,
            "cmd_expr_arg" => Token::cmd_expr_arg,
            "cmd_expr_option" => Token::cmd_expr_option,
            "_cmd_expr_arg_inner" => Token::_cmd_expr_arg_inner,
            "cmd_text_arg" => Token::cmd_text_arg,
            "_cmd_text_arg_block" => Token::_cmd_text_arg_block,
            "math_cmd" => Token::math_cmd,
            "math_cmd_name" => Token::math_cmd_name,
            "math_cmd_expr_arg" => Token::math_cmd_expr_arg,
            "math_cmd_expr_option" => Token::math_cmd_expr_option,
            "_math_cmd_expr_arg_inner" => Token::_math_cmd_expr_arg_inner,
            "literal_unit" => Token::literal_unit,
            "literal_bool" => Token::literal_bool,
            "literal_length" => Token::literal_length,
            "literal_int" => Token::literal_int,
            "literal_float" => Token::literal_float,

            // add rule
            "@require:" => Token::header_require,
            "@import:" => Token::header_import,
            s => Token::other(s.to_string()),
        }
    }
}

pub fn token_to_string(token: Token) -> String {
    token.to_string()
}

pub const LIST_EXPR: [Token; 22] = [
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

pub const LIST_RECORD_INNER: [Token; 1] = [Token::record_unit];

pub const LIST_LITERAL: [Token; 6] = [
    Token::literal_unit,
    Token::literal_bool,
    Token::literal_int,
    Token::literal_string,
    Token::literal_float,
    Token::literal_length,
];

pub const LIST_TYPE_EXPR: [Token; 9] = [
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
