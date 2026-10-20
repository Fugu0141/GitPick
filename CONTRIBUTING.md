# Contributing to GitPick

GitPick は **設計・準備段階**です。現在、動作する製品や安定した public API はありません。

## 作業の進め方

1. [Issues](https://github.com/Fugu0141/GitPick/issues) で既存タスクと依存関係を確認する。
2. 原則として **1 Issue = 1 つの目的**で作業し、必要なら先に設計を議論する。
3. 変更の理由、影響、検証方法を PR に記載する。既存ファイルの修正では関連ドキュメントも更新する。
4. 単なるファイル生成だけでなく、**追跡中のファイル、既存のルール、Windows エクスプローラーの応答性**を重点的に検証する。
5. 外部提供コードやリソースの権利・ライセンスを確認する。

## 開発の原則

- データ消失防止 > 操作回数の少なさ
- 小さな責務と小さな PR
- dry-run と approve-before-apply
- 自動変更の影響範囲が説明できること
- テストできる挙動を受け入れ条件とする
- 意図しない Git履歴書き換えやファイル削除を許さない

詳しい仕様は [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)、[docs/BEHAVIOR.md](docs/BEHAVIOR.md)、[docs/SAFETY.md](docs/SAFETY.md)、[docs/TEST_PLAN.md](docs/TEST_PLAN.md) を参照。

## Pull Request

- タイトル例: `feat(core): detect repository root`
- 本文に関連 Issue、変更内容、変更しない範囲、テスト結果、懸念事項を記載する。
- 挙動が変わる場合は仕様・テストも変更する。
- 仕様が未決の機能を根拠なく先回り実装しない。

## ライセンスの注意

現時点で `LICENSE` は未設定です。ライセンスが決まるまでは、外部からのコード提供を安易にマージしないでください。ソースの閲覧可能性は再利用許諾を意味しません。

## バグ・安全上の問題

Issue テンプレートを利用してください。ただし、認証情報、個人情報、機密のパスを含むログを公開しないでください。セキュリティ報告手順は一般公開前に別途定義します。
