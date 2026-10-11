pub mod config;
#[cfg(feature = "lsp")]
mod lsp;
mod printer;
#[cfg(test)]
mod tests;
mod visualize;

pub use config::{resolve_config, ConfigError, FormatConfig, PartialConfig};
#[cfg(feature = "lsp")]
#[allow(deprecated)]
pub use lsp::formatting;
use satysfi_parser::{grammar, CstText};
use std::fmt;
pub use visualize::*;

/// 整形できなかった理由。位置は 0 始まり。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// 構文エラー
    Parse {
        line: usize,
        column: usize,
        expected: Vec<&'static str>,
    },
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Parse { line, column, .. } => {
                write!(f, "parse error at line {}, column {}", line + 1, column + 1)
            }
        }
    }
}

impl std::error::Error for FormatError {}

pub(crate) fn parse(input: &str) -> Result<CstText, FormatError> {
    CstText::parse(input, grammar::program).map_err(|(pos, expected)| FormatError::Parse {
        line: pos.line,
        column: pos.column,
        expected,
    })
}

/// satysfi のコードを `config` に従って整形する。
///
/// 入出力 (ファイルの読み書きや設定ファイルの探索) は行わない。
/// 設定は [`resolve_config`] などで作って渡す。
/// コメントによる整形の制御 (`% satysfi-fmt: off` / `on`) に対応している。
pub fn format(input: &str, config: &FormatConfig) -> Result<String, FormatError> {
    let csttext = parse(input)?;
    let output = printer::Printer::new(&csttext, config).print();
    let mut output = printer::directive::restore_off_regions(&csttext, &output);
    // `on` のない `off` で、ファイルの最後まで元のままのとき
    if config.final_newline && !output.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}
