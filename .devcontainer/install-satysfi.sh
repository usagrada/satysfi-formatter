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

# コンテナ内では bubblewrap が使えないので sandbox は切る
opam init -y --bare --disable-sandboxing --disable-shell-hook
opam switch create satysfi "$OCAML_VERSION"
eval "$(opam env --switch=satysfi)"
opam repository add -y satysfi-external https://github.com/gfngfn/satysfi-external-repo.git
opam repository add -y satyrographos https://github.com/na4zagin3/satyrographos-repo.git
opam update
opam install -y satysfi satysfi-dist satyrographos
satyrographos install

# 以降のシェルで satysfi を使えるようにする
for rc in ~/.bashrc ~/.zshrc; do
  grep -q 'opam env --switch=satysfi' "$rc" 2>/dev/null \
    || echo 'eval "$(opam env --switch=satysfi)"' >> "$rc"
done

echo "done: 新しいシェルを開くか 'eval \$(opam env --switch=satysfi)' を実行すると satysfi が使える"
