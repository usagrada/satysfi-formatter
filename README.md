# satysfi-formatter

SATySFi 用の formatter

[document](./doc/doc.pdf)

仕様は [docs/spec.md](./docs/spec.md)、実装とテストの方針は [docs/implementation.md](./docs/implementation.md)、開発環境は [docs/development.md](./docs/development.md) にまとめている。

## install

```bash
cargo install --force --git https://github.com/usagrada/satysfi-formatter.git --branch main
```

## usage

```bash
satysfi-fmt main.saty              # 整形結果を標準出力に出す
satysfi-fmt -o out.saty main.saty  # 別のファイルに出力する
satysfi-fmt -w main.saty           # ファイルを上書きする
satysfi-fmt --check main.saty      # 整形済みか確認する (CI 向け)
```

複数のファイルは `--write` か `--check` のときに指定できる。

```bash
satysfi-fmt -w *.saty *.satyh
```

構文エラーのときは何も出力しない。

### options

| option | description |
|---|---|
| `-w`, `--write` | 入力ファイルを上書きする |
| `-o`, `--output PATH` | 出力先のファイル |
| `--check` | ファイルを書き換えず、整形されていないファイルの名前を標準エラー出力に出す |
| `--config PATH` | 設定ファイルを指定する。省略すると自動で探す ([設定ファイル](#設定ファイル)) |
| `--assume-filename PATH` | 標準入力を整形するときの仮のファイル名 ([エディタから使う](#エディタから使う)) |

### 終了コード

| コード | 意味 |
|---|---|
| 0 | 成功 (`--check` のときは、すべて整形済み) |
| 1 | `--check` で、整形されていないファイルがあった |
| 2 | エラー (構文エラー、ファイルの読み書きの失敗、設定ファイルの誤り、引数の誤り) |

複数のファイルを渡したとき、途中でエラーになっても残りは処理する。

## 設定ファイル

整形の設定は、設定ファイル `.satysfi-fmt.toml` に書く。
コマンドライン引数では変えられない。

```toml
line_width = 100      # 行幅 (既定 80)
indent_width = 2      # インデント幅 (既定 4)
use_tabs = false      # インデントにタブを使うか (既定 false)
final_newline = true  # 末尾に改行を入れるか (既定 true)
block_command_space = false  # ブロックコマンドのテキスト引数の前に空白を入れるか (既定 false)
```

* すべて省略でき、省略した項目は既定値になる。`line_width` と `indent_width` は 1 以上。
* 整形するファイルのディレクトリから親へ向かって探し、最初に見つかったものを使う。
  プロジェクトのルートに置けば、サブディレクトリのファイルにも使われる。
* `--config PATH` を指定したときは、探さずにそのファイルを使う。
* 書き間違い (知らない項目名など) はエラーになる。

各項目の意味:

* `line_width`: 行幅。リストやレコードは、この幅に収まれば 1 行に、収まらなければ縦に展開される。
  目安なので、長い識別子や文字列などは超えることがある。全角文字は 2 桁として数える。
* `block_command_space`: `true` なら `+p { ... }`、`false` なら `+p{ ... }`。
* `use_tabs`: `true` のときも、タブ 1 つを `indent_width` 桁として行幅を数える。

## エディタから使う

標準入力を整形して標準出力に出せるので、エディタや language server は、このコマンドを呼べばよい。

```bash
satysfi-fmt --assume-filename path/to/main.saty < main.saty
```

* `FILE` を省略するか `-` を指定すると、標準入力を読む。
* `--assume-filename` (別名 `--stdin-filepath`) には、整形する文書のパスを渡す。
  設定ファイルをそのパスから探すのと、エラー表示に使う。省略するとカレントディレクトリから探す。
* 標準出力には整形結果だけが出る。エラーなどは標準エラー出力に出る。
* 構文エラーは `path:line:column: error: syntax error` の形で出る (行・列は 1 始まりで、列は文字数)。

## 整形を無効にする

手で揃えた表など、整形してほしくない部分は、コメントで整形を止められる。

| コメント | 意味 |
|---|---|
| `% satysfi-fmt off` | ここから整形しない |
| `% satysfi-fmt on` | ここから整形を再開する |

整形前:

```satysfi
let x = [1;   2]
% satysfi-fmt off
let table = [
  [1;   0;   0];
  [0;   1;   0];
]
% satysfi-fmt on
let y = [1;   2]
```

整形後:

```satysfi
let x = [1; 2]
% satysfi-fmt off
let table = [
  [1;   0;   0];
  [0;   1;   0];
]
% satysfi-fmt on
let y = [1; 2]
```

### どこまで整形されなくなるか

* `off` のコメントの次から、`on` のコメントの手前までが対象。この範囲は、書いたとおりのまま出力される。
  空白・改行・空行・字下げ・コメントも変わらず、行幅を超えても折り返されない。
* `off` / `on` のコメント自体は対象に含まれず、通常どおり字下げが整えられる。
* 範囲の外は、いつもどおり整形される。
* `on` を書かなければ、ファイルの最後まで整形されない。
  `module ... struct ... end` の中に `off` を書いても、`end` の後ろも含めて最後までである。
* `off` と `on` の組は、何か所でも書ける。
  範囲の中に書いた `off` と、対応する `off` のない `on` は、何も起こらない。

### よくある使い方

ファイルの一部分だけ整形しない: `off` と `on` で挟む (上の例)。

ファイル全体を整形しない: 先頭に `off` を書く。

```satysfi
% satysfi-fmt off
let table = [
  [1;   0;   0];
  [0;   1;   0];
]
```

リストやレコードの要素の一部だけ整形しない: 要素の間にも書ける。

```satysfi
let m = [
  [1; 0];
  % satysfi-fmt off
  [0;   1];
  [5;  5];
  % satysfi-fmt on
  [2; 2];
]
```

整形後は、`off` / `on` のコメントだけ字下げが揃い、間の要素は元のままになる。

```satysfi
let m = [
    [1; 0];
    % satysfi-fmt off
  [0;   1];
  [5;  5];
    % satysfi-fmt on
    [2; 2];
]
```

縦モード・横モードの途中 (`+p { ... }` の並びや、テキストの中) にも書ける。

```satysfi
document(||)'<
  +p { a }
  % satysfi-fmt off
  +p   {  b  }
  +p   {  c  }
  % satysfi-fmt on
  +p { d }
>
```

コードの行末に書いて、その行の次から止めることもできる。
`off` を行末に書くと、その行のコードは整形され、次の行から整形されなくなる。
`on` を行末に書くと、その行のコードは元のままで、次の行から整形に戻る。

```satysfi
let   a  =  1 % satysfi-fmt off
let   b  =  2
let   c  =  3 % satysfi-fmt on
let   d  =  4
```

```satysfi
let a = 1 % satysfi-fmt off
let   b  =  2
let   c  =  3 % satysfi-fmt on
let d = 4
```

理由を `:` の後ろに書ける。

```satysfi
% satysfi-fmt off: 表を揃えるため
```

### 指示になるコメントの書き方

* `satysfi-fmt` と `off` / `on` の間には、空白が 1 つ以上要る。`%` の後ろの空白はなくてもよい。
  `%% satysfi-fmt off` のように `%` を重ねてもよい。
* 次のものは普通のコメントで、整形は止まらない。
  * `% off`
  * `% satysfi-fmt: off` (コロンを付けた書き方)
  * `% satysfi-fmt off the table` (`:` なしで文章が続くもの)
  * `% 説明 satysfi-fmt off` のように、コメントの途中に書いたもの
  * 文字列の中に書いた `satysfi-fmt off`

### 注意

* 整形しない範囲の中も、SATySFi の構文として正しい必要がある。構文エラーは、範囲の中でもエラーになる。
* 設定ファイルで、ファイルやディレクトリを整形の対象から外す機能はない。
  対象外にしたいファイルは、コマンドに渡さないか、先頭に `% satysfi-fmt off` を書く。
* 構文の途中 (`let-rec` と関数名の間など) に書くと、コメントの位置を保てないことがある。
  そのときは、`off` と `on` を含む構文のまとまり (文の並びなら、`off` を含む文から `on` を含む文まで)
  が、整形されない範囲に含まれる。行頭や行末に書く通常の使い方では、範囲は広がらない。

## 開発環境

Dev Container で開発環境を用意できる。手順は [docs/development.md](./docs/development.md) を参照。

## 謝辞

@monaqa さんの [satysfi-parser](https://github.com/monaqa/satysfi-parser) を元に format を行っている。
