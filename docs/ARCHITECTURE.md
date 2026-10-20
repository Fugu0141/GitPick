# アーキテクチャ（初期設計）

Status: **Proposed** / 2026-10-20

## 目標と設計原則

GitPick は、Windows エクスプローラーの右クリックから安全に Git の管理設定を変更するローカルツール。GitHub API やログインは基本機能に不要。

- **Shell は薄く**: メニュー表示でディスク全走査・Git 実行・ネットワーク通信をしない。
- **Core は独立**: Explorer 固有のコードから Git のルール判定を切り離す。
- **Plan と Apply を分離**: プレビューが成功して初めて適用できる。
- **Git の実際の解釈を尊重**: 独自パーサーだけで最終的な ignore / attributes 状態を決めない。
- **最小権限とローカル完結**: 管理者権限不要を目標にする。利用状況やパスを送信しない。

## コンポーネント（想定）

```text
Windows Explorer (selected shell items)
    |
    v
gitpick-shell (native C++ COM / IExplorerCommand)
    | Invoke: selected paths, requested action
    v
gitpick-app (confirmation UI / runner, separate process)
    |
    v
gitpick-core (Rust library)
    |-- repository: repository root, worktree and nested repo discovery
    |-- status: index, tracked/untracked, ignore and attributes inspection
    |-- rules: target-specific ignore / LFS rule plans
    |-- plan: warnings, affected paths, file diffs, preconditions
    |-- apply: check-then-write, restore and verification
    v
git CLI / git-lfs CLI, .gitignore, .gitattributes
```

CLI (`gitpick scan`, `gitpick plan`, `gitpick apply`) は UI とは独立して core を使う。CLI 構文は仮で、最初のプロトタイプで確定する。

### 境界のルール

1. **Shell extension**: 選択パスを受け取り、GitPick を起動。エクスプローラーをブロックしない。対象によって表示できるメニューを切り替える（ただし高負荷な判定はしない）。
2. **App / runner**: 操作を受け取り、ユーザーの確認を求め、進捗と結果を表示する。Gitコマンドを直接組み立てない。
3. **Core**: パス検証、状態判定、実行計画作成、変更検証、ファイル更新。Windows 固有 API に依存させない方針。
4. **Git adapter**: 引数配列で `git` / `git lfs` を呼び出す。シェル文字列を経由しない。結果は NUL 区切り形式で読む。バージョン・実行失敗を識別する。

設計詳細: [Git状態取得の契約](CORE_INSPECTION_DESIGN.md)（Issue #4）。

## 想定するデータ契約

`Selection`: 入力パスの一覧、希望操作（ignore / lfs / remove / inspect）。

`RepositoryContext`: Gitルート、作業ツリー、別 worktree / サブモジュールの有無、Git・LFS の利用可否。

`Plan`:
- 対象とリポジトリの識別
- 各ファイルの現在の状態（tracked / untracked / ignored / LFS / conflict）
- 変更予定ファイルと統一差分
- 影響対象・警告・操作の分類（rule-only / index-change / history）
- 検証用の変更前ハッシュ・状態（stale plan 対策）
- 何もしない場合も明示する

`ApplyResult`: 適用したファイル、残りの手作業、検証結果、エラー、復旧手段。

## Windows 11 統合

新しい右クリックメニューでは、ネイティブ COM DLL で `IExplorerCommand` を実装し、パッケージ identity と `windows.fileExplorerContextMenus` 宣言で登録する。サブメニューには `EnumSubCommands` を使用。

初期プロトタイプでは「メニュー表示→別プロセス起動」が可能かを先に検証。配布方式は **MSIX / sparse package と従来型インストーラー**を比較してから確定する。Win10 対応は非目標（後日検討）。

参照: [Microsoft Learn](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/integrate-packaged-app-with-file-explorer)。

## ディレクトリ案（コード導入時）

```text
crates/
  gitpick-core/
  gitpick-cli/
windows/
  shell-extension/
  packaging/
docs/
  ARCHITECTURE.md
  BEHAVIOR.md
  SAFETY.md
  ROADMAP.md
  TEST_PLAN.md
```

この構成は初期提案であり、まだ実際のプロジェクトファイルは存在しない。

## 設計上の未決事項

- 確認 UI の技術（ネイティブ / Tauri 等）
- C++ と Rust の境界（別プロセス IPC を基本案とする）
- Windows 配布・アップデート・アンインストール方式
- Git LFS 未インストール時の導線
- ライセンス
