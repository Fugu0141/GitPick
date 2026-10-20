# 開発環境とCLI（Issue #2）

Status: **Scaffold implemented / Core commands not implemented**

## ビルド要件

- Rust toolchain: **Rust 1.78 以上**（stable 推奨）
- Cargo
- Git（今後の実リポジトリ検査に必要）
- Git LFS（今後の LFS 統合テストのみ必要）
- 初期の core / CLI 自体には Windows SDK は不要

```sh
cargo build --workspace
cargo test --workspace
cargo run -p gitpick-cli -- --help
```

## 現在のCLI契約

```text
gitpick scan <path> [path ...]
gitpick plan <ignore|unignore|lfs|unlfs> <path>
gitpick apply <plan-file>
```

**上記は今のところ入口のみです。** Issue #4 / #5 / #6 / #7 が実装されるまで、いずれも終了コード 3 で明示的に拒否します。成功したふりをしたり、Git の作業ツリー、index、履歴、設定ファイルを変更したりしません。

- 終了コード 0: `--help` / `--version` の成功（今後は操作の成功も）
- 終了コード 1: 実行時エラー
- 終了コード 2: 引数エラー
- 終了コード 3: 未実装

`apply <plan-file>` のファイル形式は Issue #7 で設計します。この構文と現在の Rust API はまだ安定契約ではありません。

## モジュール境界

- `gitpick-cli`: 引数解釈とCLI入出力、終了コードのみ
- `gitpick-core::Request`: UIに依存しないリクエスト
- `gitpick-core::execute`: 責務を持つ処理へのルーティング。現状は失敗クローズ
- `gitpick-core::process::ProcessRunner`: 引数配列で外部コマンドを呼ぶ抽象。呼び出し側は信頼済みGit実行ファイルを選択すること。標準出力/エラー出力はバイト列で保持し、将来の `-z` 出力を壊さない

## ログ方針

- 明示的な診断なしにパス・ファイル内容・環境変数・Git出力を収集・送信しない
- 危険な操作はプレビューが完成するまで使用不可とする
- Gitコマンドの組み立てにシェル連結を使わない
- テストは使い捨てリポジトリで行い、普段の作業ツリーを直接変更しない

今後: Issue #3 で Git fixture / Windows・Linux CI を追加します。
