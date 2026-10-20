# テスト計画

Status: **Proposed** / 2026-10-20

## テスト方式

- Unit: パス→パターンの変換、ルール編集・重複判定、plan の生成。
- Integration: 一時ディレクトリに実際の `git init` を行い、Git の出力と変更結果を検証。LFS がある環境では `git lfs` を使用。
- Windows E2E: Explorer メニュー登録、単一/複数選択、アンインストール、Explorer 応答性。
- Regression: 再現した不具合は入力と期待結果を固定して残す。

## 必須ケース

| ID | 入力・条件 | 期待結果 |
| --- | --- | --- |
| G01 | Git リポジトリ外のファイル | エラー説明のみ・無変更 |
| G02 | 普通の Git リポジトリ内ファイル | 正しいルートと相対パス |
| G03 | ネストしたリポジトリ / submodule | 境界を跨いだ編集なし |
| G04 | 別 worktree / Git の管理ディレクトリ分離 | 対象 worktree を正しく識別 |
| I01 | 未追跡ファイルを除外 | ルールが追加され Git が ignore と判定 |
| I02 | 追跡済みファイルを除外 | ignore は追加できるが tracked の注意を表示 |
| I03 | 同一操作を2回実施 | 追加差分なし |
| I04 | `# ! * ? [ ]`, 空白、Unicode を含む名前 | そのパスだけを対象にできる |
| I05 | 子 `.gitignore` / negation / global ignore が存在 | 既存の規則を尊重し状態を正しく説明 |
| L01 | 未追跡の大容量ファイル | 選択ファイル向け LFS ルールを生成 |
| L02 | LFS 未インストール | 変更せず対処方法を案内 |
| L03 | tracked だが非LFSファイル | 過去履歴と index の扱いを警告 |
| L04 | LFSルールと他の属性ルールが競合 | 有効属性を再検証 |
| L05 | フォルダーにソースとバイナリが混在 | 全ファイル一括 LFS を既定にしない |
| S01 | 確認前に終了 / キャンセル | ファイル内容と index が不変 |
| S02 | プレビュー後に対象設定が変更 | stale plan として停止 |
| S03 | 書き込み途中エラー | 部分失敗を報告し復旧可能 |
| S04 | 対象が symlink/junction で外部を指す | 禁止または明示的に安全判定 |
| S05 | 既存の BOM / CRLF / コメント | 変更対象以外を維持 |
| W01 | Windows 11 現行メニュー | メニューが出る / 操作が起動 |
| W02 | 大容量フォルダー / ネットワークドライブ | メニュー生成で Explorer が固まらない |
| W03 | インストール解除後 | 右クリック項目が残らない |

## 検証の規則

- 実ファイルの削除や履歴変更を伴うテストは、必ず使い捨て Git リポジトリで実行する。
- `git status --porcelain=v1 -z`, `git ls-files -z`, `git check-ignore`, `git check-attr` 等による事後確認を必須とする。
- 1件でも内容破損や予期しない index 変更があればリリースを止める。
- CI は Windows を最低限含める。Rust core の platform 非依存部分は Linux でもテストする。

## CI / fixture 実装状況（Issue #3）

Windows / Ubuntu それぞれで `cargo fmt --all -- --check`, `cargo build --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --all-targets --locked` を実行する設計。**ジョブが開始し各ステップが成功したことを確認するまで、Issue #3 のCI受け入れ条件は未達**と扱う。

`crates/gitpick-core/tests/git_fixture.rs` は専用一時ディレクトリに `git init` する。すべての Git 呼び出しは fixture 専用の環境隔離を通し、親プロセスからの `GIT_*` 変数と明示的なGit環境変数指定を除去し、global/system Git config の読み込みを抑止する。テストで `GIT_DIR` や `GIT_INDEX_FILE` 等を意図的に汚染しても他のfixtureを変更しないことを検証する。

### Fixture のテストIDと製品機能テストの対応

| Fixture ID | テスト関数 | 検証範囲 | 将来の製品テスト（**未合格**） |
| --- | --- | --- | --- |
| F01 | `git_fixture_resolves_repository_and_preserves_special_names` | 本物のGitでUTF-8パスと `ls-files -z` を利用できる | G02, I04 |
| F02 | `git_fixture_checks_ignore_without_touching_file_content` | Git標準のignore評価とファイル保持 | I01, I05 |
| F03 | `git_fixture_can_detect_a_non_repository` | 非リポジトリからのGit応答を検証 | G01 |
| F04 | `git_fixture_isolated_from_injected_git_environment` | 汚染されたGit環境でも他のテスト用repo/indexに影響しない | G03, S01 |
| F05 | `git_fixture_lfs_tracks_selected_file_when_available` | `git lfs version` 成功環境のみ `git lfs track --filename` と `git check-attr` を検証 | L01, L02, L04 |

**Fxxはテスト基盤そのものの検証ID**であり、Gxx / Ixx / Lxx / Sxx の GitPick 機能が実装・合格した意味ではない。特に F05 は Git LFS がない環境ではスキップする。CIで LFS が利用可能な環境を必須にする前に、導入有無とスキップ状況を確認する。

本PRでIssue #3のLFS optional fixtureを用意する方針へ変更した。機能実装固有の LFS 検査（L01〜L04）はIssue #6で追加する。
