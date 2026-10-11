# satysfi-formatter

SATySFi 用の formatter

[document](./doc/doc.pdf)

## install

```bash
cargo install --force --git https://github.com/usagrada/satysfi-formatter.git --branch main
```

## usage

```bash
satysfi-fmt $input -o $output
```

## options

| option | default | description |
|---|---|---|
| `-i`, `--indent-space` | 4 | インデント幅 |
| `--line-width` | 80 | 行幅。リストやレコードは、この幅に収まれば 1 行に、収まらなければ縦に展開される |
| `--use-tabs[=BOOL]` | false | インデントにタブを使う。タブ 1 つをインデント幅の桁として行幅を数える |
| `--cspace[=BOOL]` | true | コマンドとテキスト引数の間に空白を入れる (`+p { ... }`)。`--cspace=false` で `+p{ ... }` |
| `--config` | | 設定ファイル。省略すると入力ファイルのディレクトリから親へ向かって探す |
| `-w`, `--write` | | 入力ファイルを上書きする |
| `-o`, `--output` | | 出力先のファイル |

構文エラーのときは何も出力せず、終了コード 1 で終わる。

## 設定ファイル

`.satysfi-fmt.toml` に設定を書ける。
整形するファイルのディレクトリから親へ向かって探し、最初に見つかったものを使う。

```toml
line_width = 100      # 行幅 (既定 80)
indent_width = 2      # インデント幅 (既定 4)
use_tabs = false      # インデントにタブを使うか (既定 false)
final_newline = true  # 末尾に改行を入れるか (既定 true)
command_space = true  # コマンドとテキスト引数の間に空白を入れるか (既定 true)
```

設定は次の順で優先される。

* CLI: コマンドライン引数 → 設定ファイル → 既定値
* language server: 設定ファイル → `FormattingOptions` (`tabSize` など) → 既定値

## ライブラリとして使う

一般的なフォーマッタ (Ruff、StyLua、Prettier など) と同じく、
「設定を解決する関数」と「文字列を整形する純粋な関数」を提供する。
LSP の型への変換 (`FormattingOptions` の読み取り、`TextEdit` の作成) は language server 側で行う。

* `resolve_config(&Path) -> Result<PartialConfig, ConfigError>`:
  整形するファイルのディレクトリから親へ向かって `.satysfi-fmt.toml` を探して読む。
  見つからなければ空の設定を返す。呼ぶたびに読み直すので、設定ファイルの変更はすぐ反映される。
* `PartialConfig::from_editor_options(tab_size, insert_spaces, insert_final_newline)`:
  エディタの指定 (LSP の `FormattingOptions` の値) から設定を作る。
* `PartialConfig::or` / `PartialConfig::build`: 設定を重ね、足りない項目を既定値で埋める。
* `format(&str, &FormatConfig) -> Result<String, FormatError>`:
  文字列を整形する。入出力は行わない。構文エラーは位置を含む `Err` になる。

language server からは、例えば次のように使う (設定ファイル → `FormattingOptions` → 既定値の順で優先)。

```rust
use satysfi_formatter::{config::PartialConfig, resolve_config};

let request = PartialConfig::from_editor_options(
    options.tab_size,
    options.insert_spaces,
    options.insert_final_newline,
);
let file = match uri.to_file_path() {
    Ok(path) => resolve_config(&path)?,
    Err(()) => PartialConfig::default(), // 保存前の文書など
};
let new_text = satysfi_formatter::format(&text, &file.or(request).build())?;
// new_text で文書全体 ((0, 0) から末尾まで) を置き換える TextEdit を返す
```

旧 API の `formatting(&str, FormattingOptions) -> Vec<TextEdit>` は、satysfi-language-server が移行するまでの互換として
既定で有効な feature `lsp` で残している (非推奨)。`default-features = false` にすると lspower に依存しない。

行幅は目安で、長い識別子や文字列などは行幅を超えることがある。全角文字は 2 桁として数える。

## 整形の制御

clang-format の `// clang-format off` / `// clang-format on` と同じように、コメントで整形しない範囲を指定できる。

整形前:

```satysfi
let x = [1;   2]
% satysfi-fmt: off
let table = [
  [1;   0;   0];
  [0;   1;   0];
]
% satysfi-fmt: on
let y = [1;   2]
```

整形後:

```satysfi
let x = [1; 2]
% satysfi-fmt: off
let table = [
  [1;   0;   0];
  [0;   1;   0];
]
% satysfi-fmt: on
let y = [1; 2]
```

* 指示として扱うのは、`% satysfi-fmt: off` と `% satysfi-fmt: on` の形のコメントだけ (`%` は 1 つ、空白は自由)。
  `% off` や `% satysfi-fmt: off the table`、`%% satysfi-fmt: off`、コメントの途中に書いたもの、
  文字列やヘッダ行の中の同じ文字列は、普通のコメント・文字列として扱い、整形を止めない。
* `% satysfi-fmt: off` から `% satysfi-fmt: on` までは、空白・改行・字下げ・コメントも含めて元のまま出力する。
* `off` / `on` のコメント自体は、通常どおり字下げされる。
* 文の間だけでなく、リストやレコードの要素の間、`+p { ... }` の中など、式やブロックの途中にも書ける。
* 行末に書いた `off` はその次の行から、行末に書いた `on` はその次の行から整形を再開する
  (行末の `on` より前は元のまま)。
* `on` がなければ、ファイルの最後まで整形しない。
* `% satysfi-fmt: off: 表を揃えるため` のように、`:` の後ろに理由を書ける。
* 構文の途中 (`let-rec` と関数名の間など) に書いたときは、`off` / `on` のコメントの位置は変えずに、
  整形しない範囲を広げる。`off` と `on` を含む同じ階層のノードの並び (文の並びなら、`off` を含む文から
  `on` を含む文まで) を丸ごと元のまま出力し、それでも位置が変わるなら 1 つ上の階層へ広げる。
  最後はファイル全体を元のまま出力するので、エラーにはならない。
  行頭や行末に書く普通の使い方では、広がらない。

@monaqa さんの [satysfi-parser](https://github.com/monaqa/satysfi-parser) を元にformatを行っている
