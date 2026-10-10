//! 旧 API (lspower の型を使う)。satysfi-language-server が新しい API に移行するまでの互換のために残す。
//!
//! 新しいコードでは [`crate::format`] と [`crate::config::resolve_config`] を使う。

use crate::{format, PartialConfig};
use lspower::lsp::{FormattingOptions, FormattingProperty, Position, Range, TextEdit};

/// 文書全体を整形し、置き換える `TextEdit` を返す。構文エラーのときは空を返す。
///
/// `FormattingOptions` だけを使い、設定ファイルは読まない。
#[deprecated(note = "use `satysfi_formatter::format` with `config::resolve_config` instead")]
pub fn formatting(input: &str, option: FormattingOptions) -> Vec<TextEdit> {
    let line_width = match option.properties.get("lineWidth") {
        Some(FormattingProperty::Number(n)) if *n > 0 => Some(*n as usize),
        _ => None,
    };
    let config = PartialConfig {
        line_width,
        ..PartialConfig::from_editor_options(
            option.tab_size,
            option.insert_spaces,
            option.insert_final_newline,
        )
    }
    .build();
    match format(input, &config) {
        Ok(new_text) => {
            let line = input.matches('\n').count() as u32;
            let last = input.rsplit('\n').next().unwrap_or("");
            vec![TextEdit {
                range: Range {
                    start: Position::new(0, 0),
                    end: Position::new(line, last.encode_utf16().count() as u32),
                },
                new_text,
            }]
        }
        Err(err) => {
            eprintln!("disable to format\n[{}]", err);
            Vec::new()
        }
    }
}
