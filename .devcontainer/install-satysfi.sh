#!/usr/bin/env bash
# satysfi コマンドが必要なとき (cargo make build-doc など) に、コンテナ内で手動で実行する。
# opam で OCaml と SATySFi をビルドするので、初回は時間がかかる。
set -euo pipefail

OCAML_VERSION="${OCAML_VERSION:-4.14.2}"
export OPAMCONFIRMLEVEL=unsafe-yes

sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  build-essential git m4 unzip curl wget pkg-config bzip2 opam \
  libpng-dev libjpeg-dev libfreetype-dev libgmp-dev

# 何度実行しても同じ状態になるよう、各ステップは済んでいれば飛ばす
# コンテナ内では bubblewrap が使えないので sandbox は切る
if [ ! -d "$HOME/.opam" ]; then
  opam init -y --bare --disable-sandboxing --disable-shell-hook
fi
if ! opam switch list --short | grep -qx satysfi; then
  opam switch create satysfi "$OCAML_VERSION"
fi
eval "$(opam env --switch=satysfi)"
if ! opam repository list --short | grep -qx satysfi-external; then
  opam repository add -y satysfi-external https://github.com/gfngfn/satysfi-external-repo.git
fi
if ! opam repository list --short | grep -qx satyrographos; then
  opam repository add -y satyrographos https://github.com/na4zagin3/satyrographos-repo.git
fi
opam update
opam install -y satysfi satysfi-dist satyrographos
# To build doc.saty
if ! opam pin list --short | grep -q '^satysfi-code-printer'; then
  opam pin add -y satysfi-code-printer "git+https://github.com/puripuri2100/satysfi-code-printer.git"
fi
opam install -y satysfi-code-printer

# satyrographos は ~/.satysfi/dist を自分で管理する。過去のシンボリックリンクは先に外す
# (外さないと、リンク先の opam 側 dist を消してしまう)
if [ -L "$HOME/.satysfi" ]; then rm "$HOME/.satysfi"; fi
# 管理済みならそのまま上書きされる。post-create.sh が置いた dist などで
# "not managed by Satyrographos" と断られたときだけ消して入れ直す
if ! satyrographos install; then
  rm -rf "$HOME/.satysfi/dist"
  satyrographos install
fi
test -f "$HOME/.satysfi/dist/hash/default-font.satysfi-hash"

# 以降のシェルで satysfi を使えるようにする
for rc in ~/.bashrc ~/.zshrc; do
  grep -q 'opam env --switch=satysfi' "$rc" 2>/dev/null \
    || echo 'eval "$(opam env --switch=satysfi)"' >> "$rc"
done

echo "done: 新しいシェルを開くか 'eval \$(opam env --switch=satysfi)' を実行すると satysfi が使える"
