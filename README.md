# GitPick

**Manage Git with a Click.**  
Windows エクスプローラーの右クリックから、ファイルやフォルダーの Git 管理方法を選べるようにするツールです。

> [!IMPORTANT]
> **開発準備中 / Planning stage** — まだ動作するアプリ、インストーラー、リリースはありません。このリポジトリは初期設計を公開・管理するためのものです。

## コンセプト

GitPick は、`.gitignore` や `.gitattributes` を毎回手で編集する手間を減らします。リポジトリ内のファイル・フォルダーを右クリックし、**Git の管理対象から除外**するか、**Git LFS で管理**するかを選ぶことが目標です。

### 目指す操作体験

1. Git リポジトリ内のファイルまたはフォルダーを右クリックする。
2. **GitPick** → **Git 管理から除外 / Git LFS で管理 / 状態を確認** を選ぶ。
3. 影響範囲、既存の設定、変更差分、注意点を確認する。
4. 承認した内容だけがローカルに反映される。

基本方針は **「ワンクリックで候補を選び、影響のある変更は確認して適用」** です。Git の履歴やファイル本体を勝手に書き換えることはしません。

## 対象プラットフォーム・構成（予定）

- **Windows 11**: 新しいエクスプローラー右クリックメニューを最優先
- **Shell extension**: ネイティブ C++ / COM `IExplorerCommand`
- **Git 操作エンジン**: Rust 製コアと CLI
- **統合方法**: 軽量なシェル拡張が外部プロセスを起動し、重い処理はシェルの外で実行
- **前提**: ローカルの Git。Git LFS 操作には `git-lfs` のインストールが必要

この技術選定は初期方針であり、プロトタイプの検証結果に応じて調整します。

## 主な機能（計画）

| 機能 | 最初の目標 |
| --- | --- |
| Git 管理から除外 | 対象に対応する `.gitignore` ルールの追加・確認 |
| Git LFS 指定 | `.gitattributes` に対象ファイル用の LFS ルールを設定 |
| 現在の状態を確認 | 追跡済み / 未追跡 / ignore / LFS 適用状態を確認 |
| プレビュー | 変更予定のルール、対象数、注意点を適用前に表示 |
| 設定解除 | GitPick が追加したルールを安全に取り消す |
| 複数選択・フォルダー | 影響範囲を表示し、必要に応じて個別指定（段階導入） |
| リポジトリ解析 | 不要ファイルや大容量ファイルを推奨する機能（将来） |

### 大容量ファイルについて

GitHub の通常の Git 管理では **50 MiB 超で警告、100 MiB 超でブロック**されます。Git LFS は通常の Git オブジェクトの代わりにポインタを保持し、実データを別途管理します。GitPick は LFS 設定を手助けしますが、サイズだけを条件に `.gitattributes` を書くことはできません。また、過去のコミットを自動的に変換することもありません。

- 既に Git が追跡しているファイルは、`.gitignore` に書いただけでは追跡解除されません。
- 既にコミット済みの大容量ファイルは、LFS ルールを加えただけでは過去の履歴から消えません。
- GitPick は **ファイル削除、commit、push、履歴書き換えを自動実行しない** 方針です。

詳しくは [操作仕様](docs/BEHAVIOR.md) と [安全設計](docs/SAFETY.md) を参照してください。

## ドキュメント

- [アーキテクチャ](docs/ARCHITECTURE.md) — コンポーネントと責務の分離
- [操作仕様](docs/BEHAVIOR.md) — 右クリック操作、Git の状態、例外
- [安全設計](docs/SAFETY.md) — 破壊的操作を防ぐための要件
- [ロードマップ](docs/ROADMAP.md) — 開発フェーズと実装順
- [テスト計画](docs/TEST_PLAN.md) — 振る舞いとエッジケースの検証
- [開発への参加](CONTRIBUTING.md) — Issue・PR の進め方

## 開発状況

設計・Issue の分解を進めています。今はダウンロードして実行できる状態ではありません。実装の進行は [Issues](https://github.com/Fugu0141/GitPick/issues) で確認できます。

## 参考資料

- [Microsoft: File Explorer context menu command](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/integrate-packaged-app-with-file-explorer)
- [Git: gitignore](https://git-scm.com/docs/gitignore)
- [Git: gitattributes](https://git-scm.com/docs/gitattributes)
- [Git LFS: FAQ](https://github.com/git-lfs/git-lfs/blob/main/docs/man/git-lfs-faq.adoc)
- [GitHub: About large files](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-large-files-on-github)

## ライセンス

**未決定**。ライセンスを選定するまでは、明示的なオープンソースライセンスによる利用許諾はありません。公開配布や外部コードの取り込みを始める前に決定します。
