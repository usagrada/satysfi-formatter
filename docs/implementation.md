# satysfi-formatter 実装方針・テスト方針 (草案)

> 仕様は [spec.md](./spec.md) を参照。本書は実装とテストの進め方をまとめる。
> `feature/support-pretty-format` ブランチ (`5a8c227` 時点) の内容に基づく。

## 1. 実装方針

1. `pretty` crate (0.12) の `RcDoc` で CST を Doc に変換 (lower) し、行幅を指定して描画する (`src/printer/`)。
2. rule の種類ごとにモジュールを分ける: `expr` (式)、`stmt` (文)、`types` (型)、`pattern` (パターン)、`text` (垂直/水平/数式モード)、`directive` (整形の制御。いったん全体を整形してから、`off` と `on` の間を整形前の文字列に戻す)。
3. コメントと空行は `CstText::items` で子ノードの隙間から取り出し、Doc に埋め込む。行コメントの後ろには必ず `hardline` を置く。
4. `main.rs` は薄い CLI にし、整形の設定は設定ファイルだけから受け取る。標準入力に対応し、エディタからプロセスとして呼べるようにする。
5. 設定は `src/config.rs` (`FormatConfig` / `PartialConfig` / `.satysfi-fmt.toml`) に分け、`format` は純粋関数にする。LSP 型への依存は feature `lsp` (`src/lsp.rs`) に閉じ込める。
6. 旧 `Formatter` (`format_legacy`) と、それを支える `comment.rs` などは削除済み。

## 2. テスト方針

- `src/tests/*.rs`: 入力と期待出力の組による単体テスト。rule ごと (`exprs`, `stmts`, `types`, `patterns`, `text_mode`, `math_mode`)、コメントの位置 (`comment_position`)、directive (`directive`)、オプションと設定 (`options`) に分ける。
- `src/tests/printer.rs`: `examples/*.saty` と `doc/*.saty` を複数の行幅で整形し、冪等性と整形前後のトークン保存を検証する。
- `tests/cli.rs`: CLI を実際に起動し、標準入出力・終了コード・設定ファイルの探索を確認する。
- `tests/build-test.py`: 整形後のコードが SATySFi でビルドできることの確認用。
