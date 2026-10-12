# 開発環境

## Dev Container

VS Code の Dev Containers (または GitHub Codespaces) で開くと、Rust とテストに必要な SATySFi の標準パッケージが自動で用意される。
設定は [.devcontainer](../.devcontainer) にある。

1. Docker を用意する。
2. VS Code に拡張機能「Dev Containers」を入れる。
3. リポジトリを開き、コマンドパレットから `Dev Containers: Reopen in Container` を実行する。

コンテナを作ったあと ([.devcontainer/post-create.sh](../.devcontainer/post-create.sh)) に、次のことをしている。

* SATySFi の標準パッケージを `~/.satysfi/dist/packages` に置く (テストが参照する。CI と同じ手順)
* `cargo-make` を入れる (`Makefile.toml` のタスク用)
* `cargo fetch` で依存を取得する

テストは `cargo test` で実行する。

## satysfi コマンド

`satysfi` コマンドは、ドキュメントの生成 (`cargo make build-doc`) などでだけ必要で、`cargo test` には要らない。
そのため、コンテナには入れていない。必要なときに、コンテナ内で次を実行する。

```bash
bash .devcontainer/install-satysfi.sh
```

opam で OCaml と SATySFi をビルドするので、初回は時間がかかる。
コンテナを作り直すと消えるので、そのときは再度実行する。
