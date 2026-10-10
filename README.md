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
| `-w`, `--write` | | 入力ファイルを上書きする |
| `-o`, `--output` | | 出力先のファイル |

## 整形の制御

コメントで、整形しない範囲を指定できる。整形しないノードは元の文字列のまま出力される (ノード間の空白や改行は通常どおり整形される)。

```satysfi
% satysfi-fmt: skip
let table = [
  [1;   0;   0];
  [0;   1;   0];
]

let x = [1;   2]   % satysfi-fmt: skip

% satysfi-fmt: off
let a   =   1
let b   =   2
% satysfi-fmt: on
```

* `% satysfi-fmt: skip` : 直後のノードを整形しない。行末に書いた場合は、その行のノード。
* `% satysfi-fmt: off` / `% satysfi-fmt: on` : 間にあるノードを整形しない。`on` がなければ、同じ階層の最後のノードまで。

@monaqa さんの [satysfi-parser](https://github.com/monaqa/satysfi-parser) を元にformatを行っている
