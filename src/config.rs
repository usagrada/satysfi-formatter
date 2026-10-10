//! 整形の設定。
//!
//! 設定は次の順で重ねる (上ほど優先)。
//!
//! * CLI: コマンドライン引数 → 設定ファイル → 既定値
//! * language server: 設定ファイル → `FormattingOptions` (`tabSize` など) → 既定値
//!
//! 呼び出し側 (CLI / language server) は、文書のパスを [`resolve_config`] に渡して設定ファイルを読み、
//! 自分の持つ設定 (コマンドライン引数やエディタの指定) と重ねてから [`crate::format`] に渡す。
//!
//! ```ignore
//! // language server の例
//! let request = PartialConfig::from_editor_options(
//!     options.tab_size,
//!     options.insert_spaces,
//!     options.insert_final_newline,
//! );
//! let config = resolve_config(&path)?.or(request).build();
//! let new_text = satysfi_formatter::format(&text, &config)?;
//! ```
//!
//! 設定ファイル (`.satysfi-fmt.toml`) は、
//! 整形するファイルのディレクトリから親へ向かって探す。→ [`find_config_file`]
//!
//! ```toml
//! line_width = 100
//! indent_width = 2
//! use_tabs = false
//! final_newline = true
//! command_space = true
//! ```

use serde::Deserialize;
use std::fmt;
use std::path::{Path, PathBuf};

/// 設定ファイルの名前。
pub const CONFIG_FILE_NAME: &str = ".satysfi-fmt.toml";

/// 整形の設定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatConfig {
    /// 行幅。リストやレコードは、この幅に収まれば 1 行に、収まらなければ縦に展開される
    pub line_width: usize,
    /// インデント幅。`use_tabs` のときは、タブ 1 つをこの桁数として行幅を数える
    pub indent_width: usize,
    /// インデントにタブを使うか
    pub use_tabs: bool,
    /// 末尾に改行を入れるか
    pub final_newline: bool,
    /// コマンドとテキスト引数の間に空白を入れるか (`+p { ... }` / `+p{ ... }`)
    pub command_space: bool,
}

impl FormatConfig {
    pub const DEFAULT_LINE_WIDTH: usize = 80;
    pub const DEFAULT_INDENT_WIDTH: usize = 4;
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            line_width: Self::DEFAULT_LINE_WIDTH,
            indent_width: Self::DEFAULT_INDENT_WIDTH,
            use_tabs: false,
            final_newline: true,
            command_space: true,
        }
    }
}

/// 一部だけが指定された設定。[`PartialConfig::or`] で重ね、[`PartialConfig::build`] で既定値を埋める。
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartialConfig {
    pub line_width: Option<usize>,
    pub indent_width: Option<usize>,
    pub use_tabs: Option<bool>,
    pub final_newline: Option<bool>,
    pub command_space: Option<bool>,
}

impl PartialConfig {
    /// `self` で指定されていない項目を `lower` で補う。
    pub fn or(self, lower: PartialConfig) -> PartialConfig {
        PartialConfig {
            line_width: self.line_width.or(lower.line_width),
            indent_width: self.indent_width.or(lower.indent_width),
            use_tabs: self.use_tabs.or(lower.use_tabs),
            final_newline: self.final_newline.or(lower.final_newline),
            command_space: self.command_space.or(lower.command_space),
        }
    }

    /// 指定されていない項目を既定値で埋める。
    pub fn build(self) -> FormatConfig {
        let default = FormatConfig::default();
        FormatConfig {
            line_width: self.line_width.unwrap_or(default.line_width),
            indent_width: self.indent_width.unwrap_or(default.indent_width),
            use_tabs: self.use_tabs.unwrap_or(default.use_tabs),
            final_newline: self.final_newline.unwrap_or(default.final_newline),
            command_space: self.command_space.unwrap_or(default.command_space),
        }
    }

    /// TOML の文字列から読む。
    pub fn from_toml(text: &str) -> Result<Self, ConfigError> {
        let config: PartialConfig =
            toml::from_str(text).map_err(|err| ConfigError::Parse(err.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    /// 設定ファイルを読む。
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path).map_err(|err| ConfigError::Io {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?;
        Self::from_toml(&text).map_err(|err| err.in_file(path))
    }

    /// エディタから渡される整形の指定 (LSP の `FormattingOptions` の
    /// `tabSize` / `insertSpaces` / `insertFinalNewline`) から作る。
    ///
    /// `tabSize` はエディタでのタブの表示幅なので、設定ファイルやエディタの設定より優先度を下げて使う。
    /// `tab_size` が 0 なら無視する。
    pub fn from_editor_options(
        tab_size: u32,
        insert_spaces: bool,
        insert_final_newline: Option<bool>,
    ) -> Self {
        PartialConfig {
            indent_width: (tab_size > 0).then_some(tab_size as usize),
            use_tabs: Some(!insert_spaces),
            final_newline: insert_final_newline,
            ..Default::default()
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.line_width == Some(0) {
            return Err(ConfigError::Invalid("line_width must be positive".into()));
        }
        if self.indent_width == Some(0) {
            return Err(ConfigError::Invalid("indent_width must be positive".into()));
        }
        Ok(())
    }
}

/// `path` (ファイルまたはディレクトリ) から親へ向かって設定ファイルを探す。
pub fn find_config_file(path: &Path) -> Option<PathBuf> {
    let start = if path.is_dir() { path } else { path.parent()? };
    start
        .ancestors()
        .map(|dir| dir.join(CONFIG_FILE_NAME))
        .find(|file| file.is_file())
}

/// `path` (整形するファイル) に対応する設定ファイルを探して読む。見つからなければ空の設定を返す。
///
/// 呼ぶたびに読み直す。
pub fn resolve_config(path: &Path) -> Result<PartialConfig, ConfigError> {
    match find_config_file(path) {
        Some(file) => PartialConfig::from_file(&file),
        None => Ok(PartialConfig::default()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Io {
        path: PathBuf,
        message: String,
    },
    Parse(String),
    Invalid(String),
    InFile {
        path: PathBuf,
        error: Box<ConfigError>,
    },
}

impl ConfigError {
    fn in_file(self, path: &Path) -> Self {
        ConfigError::InFile {
            path: path.to_path_buf(),
            error: Box::new(self),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io { path, message } => write!(f, "{}: {}", path.display(), message),
            ConfigError::Parse(message) => write!(f, "invalid config: {}", message),
            ConfigError::Invalid(message) => write!(f, "invalid config: {}", message),
            ConfigError::InFile { path, error } => write!(f, "{}: {}", path.display(), error),
        }
    }
}

impl std::error::Error for ConfigError {}
