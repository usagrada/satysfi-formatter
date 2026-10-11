#!/usr/bin/env bash
# テストが参照する SATySFi の標準パッケージを ~/.satysfi/dist/packages に置く (CI と同じ手順)
# satysfi 本体は必要なときに .devcontainer/install-satysfi.sh で入れる
set -euo pipefail

SATYSFI_SRC="$HOME/satysfi"
PACKAGES_DIR="$HOME/.satysfi/dist/packages"

if [ ! -d "$SATYSFI_SRC" ]; then
  git clone --depth 1 https://github.com/gfngfn/SATySFi.git "$SATYSFI_SRC"
fi
mkdir -p "$PACKAGES_DIR"
cp "$SATYSFI_SRC"/lib-satysfi/dist/packages/* "$PACKAGES_DIR/"

# Makefile.toml のタスク用
cargo install --locked cargo-make || true

# 依存を先に取得しておく
cargo fetch
