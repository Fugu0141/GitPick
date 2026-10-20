# Gitリポジトリと選択対象の状態取得 — 設計契約

Status: **Proposed / Issue #4 design** (2026-10-20)

> この文書は Issue #4 の実装前提を定義する。まだ機能実装・G01〜G04 の合格を主張しない。安全な状態取得が実装されるまで CLI `scan` は未実装エラーを返す。

## スコープと原則

対象はローカルの既存パス（Windows Explorer が選択したファイル/フォルダー）である。**読み取り操作しか許可しない**。この段階で ignore / LFS 設定・Git index・履歴の変更、ディレクトリの再帰的解析、コマンドの自動修復は行わない。

- 外部コマンドの実行は `gitpick-core::process` 経由。シェル文字列結合は禁止
- Git が実際に解釈した結果を採用し、`.gitignore` と `.gitattributes` を独自パースして推測しない
- `tracked` と `ignored` は**別の軸**。tracked file に ignore パターンが一致しても「Git追跡が解除された」とはしない
- 検査の途中で状態が変わる可能性があるため、これは変更の根拠となる不変スナップショットではない（Apply 時の再検証は #7）

## 責務と公開型（提案）

```text
SelectionGuard
  OSパス検証、再解析ポイント／リンク境界の判定
       ↓
RepositoryLocator
  リポジトリ探索、worktree・gitdir・相対パス取得
       ↓
GitStatusReader
  index / worktree / ignore / attributes を分けて問い合わせ
       ↓
StateAssembler
  項目単位に型付き結果、警告、検査エラーを統合
       ↓
scan() -> Vec<SelectionInspection> （順序は入力順、リポジトリ別にグループ化可能）
```

`SelectionInspection` が保持する候補:

| 項目 | 内容 |
| --- | --- |
| `requested_path` | OS の入力パス（CLI/GUI表示では必要に応じて伏せる） |
| `repo_root` | 対象の worktree ルート |
| `git_dir` | 対応する Git 管理ディレクトリ（`.git` をディレクトリと仮定しない） |
| `relative_path` | Git ルート相対、Git 操作用のパス |
| `kind` | file / directory（リンク等の拒否理由を含む） |
| `tracking` | **ファイルのみ**: untracked / tracked / unmerged / unknown。フォルダーは not-applicable（Git自体はディレクトリを追跡しない） |
| `index_stages` | stage と mode の情報。サブモジュールの mode `160000` を区別 |
| `worktree_change` | **ファイルのみ**: porcelain の staged / unstaged / rename 等。フォルダーは not-scanned |
| `ignore` | excluded / reincluded / non-matching / failed、根拠のルール・元ファイル・行番号、および追跡状態と独立の除外候補フラグ |
| `attributes` | 実際の filter / diff / merge / text 属性と値。フォルダー配下の属性は集約しない |
| `descendants` | フォルダーの配下集約状態。MVPでは必ず `not-scanned`（`mixed` や件数を推測しない） |
| `warnings` | tracked, LFS 未利用、境界が曖昧等の注意 |

`ignored=true` と `tracked=true` が同時に成立する場合がある。UI は1つの単純なステータスに潰さない。LFS属性の有効性と LFS バイナリの導入可否も別々に返す。


## ignore 判定の厳密な写像（レビュー #17）

`git check-ignore --stdin -z -v -n --no-index` の結果は **最後に有効だったマッチルールの情報**であり、単純に「ルールがあるので ignored=true」にはしない。`--no-index` により追跡中のパスについても除外規則を調査できるが、**Git がすでに追跡している事実は変わらない**。

| 受け取った4フィールドと終了状態 | `IgnoreDecision`（提案） | `would_ignore_if_untracked` |
| --- | --- | --- |
| 正常応答で `source`, `line`, `pattern` の3つが空 | `NotMatched` | `false` |
| 正常応答で有効パターンが `!` から始まる（エスケープ `\\!` は除外） | `Reincluded { source, line, pattern }` | `false` |
| 正常応答で有効パターンが通常の除外ルール | `Excluded { source, line, pattern }` | `true` |
| 非想定終了コード、入出力失敗、フィールド不完全、問い合わせ件数不一致 | `Failed { reason }` | `None`（不明） |

- **非一致**と**否定パターン一致**を型として分離する。前者は理由なし、後者は優先ルールの証拠を保持する。
- 否定パターンは未追跡パスの最終的な除外判断を解除する。ただし親ディレクトリそのものが除外されている場合、子の再包含が効かないことがある。実際の Git 出力を優先する。
- `would_ignore_if_untracked` は Git ignore 設定の解釈であり、**実際にGitで追跡されるかは `tracking` を参照**。tracked ならプレビューで「ignoreに一致するが追跡済み」を警告する。
- `-n` による正規の非一致（空フィールド）とコマンド失敗を混同しない。パーサが不明なデータを受けたら安全側に倒す。
- テスト（I05拡張）: `*.log` が `Excluded`、`!important.log` が `Reincluded`、`other.txt` が `NotMatched`、`foo/` が除外された時に `foo/child.txt` が `Excluded`、そして Git プロセス失敗は `Failed`。各パスは NUL 区切りで実Gitの出力と照合する。

## フォルダーの状態契約と処理予算（レビュー #17）

Git は空でないフォルダーでさえ **ディレクトリ自身を index に格納しない**。したがってフォルダーに単一の tracked/untracked 状態を与えず、選択エントリと配下集約を別物として扱う。

- `kind=directory` の時、`tracking=not-applicable`、`worktree_change=not-scanned`、`descendants=not-scanned` を返す。配下の tracked/ignored の混在状態、件数、容量は **MVP で計算しない**（将来の #12 で `mixed`, `complete`, `partial`, `unknown` 等を導入する）。
- フォルダー選択で実行してよいのは、パス境界とリポジトリ所属の確認、および **選択フォルダー自身に対する** ignore 規則の問い合わせだけ。フォルダーに `git status --untracked-files=all` を投げることは禁止。index 列挙も配下件数取得目的には使わない。
- 初期 `scan` は「選択した項目」のみ処理し、内部再帰の深さ **0**、配下ファイル列挙 **0件** とする。最大 **64項目/呼び出し**、1 Git 子プロセス **5秒**、stdout/stderr 各 **1 MiB** を暫定上限として設計。超過は `Incomplete` / `Failed` で返し、選択外や部分情報を成功と見なさない。タイムアウト時は子プロセス終了・回収が必要。負荷上限は実機計測後に調整する。
- `git rev-parse` 等でも共有ドライブなどで停止し得るので、プロセス単位の上限・キャンセルを共通基盤で扱う。stdin送信とstdout/stderr読取りを同時に行いデッドロックを避ける。
- サブモジュール、リンク、junction、UNC等で境界が不明なら全子孫の状態を断定せず拒否する。#12 のフォルダー展開・容量集計は承認画面とキャンセル機構が揃ってから有効化する。
- テスト: フォルダー内で `tracked`, `untracked`, `ignored` が混在しても `descendants=not-scanned`、巨大フォルダーでも再帰列挙が走らないこと、64件超・時間/出力超過の安全側終了を確認する。

## Git 呼び出しの規則

1. 対象パスの **親ディレクトリ**（フォルダー選択時はそのディレクトリ）を起点に Git のルートと bare/worktree 状態を確認する。選択パスの正規化・所属境界の再検証は必須
2. `git rev-parse --show-toplevel`, `--absolute-git-dir`, `--is-bare-repository`, `--show-superproject-working-tree` 等を用途別に確認する。失敗を別リポジトリの成功結果で上書きしない
3. index のエントリと merge stage は `git ls-files --stage -z -- <literal pathspec>` を基礎とする。パスのスペース・改行を行単位で分割しない
4. **ファイル選択に限り** worktree の変更は `git status --porcelain=v1 -z --untracked-files=all -- <literal pathspec>` で判定する。rename/copy 時は NUL 区切りの**追加パス要素**を正しく読む。**フォルダーを引数に渡して配下を暗黙走査させない**
5. ignore は `git check-ignore --stdin -z -v -n --no-index` へ **NUL 終端のパスを stdin で渡して**取得する。出力は `source, line, pattern, path` の4フィールド（各NUL区切り）。**返り値、4フィールドの完全性、最後の NUL、問い合わせパスとの対応を検証**する。非一致と否定ルールの詳細は後述
6. attributes は `git check-attr -z filter diff merge text -- <path>` などで確認する。`filter=lfs` の設定と `git lfs` のインストールは別判定
7. index/status のパス指定は `--` に加えて **literal pathspec** を使用する。先頭`-`や`[`等を特別扱いしない。命令ごとに literal 指定対応を検証する
8. コマンドは stdout/stderr をバイト列として扱う。Exit code、実行不能、プロトコル解析不能は区別する。ユーザーパスを診断ログに無断保存しない
9. 読み取り専用性のため `GIT_OPTIONAL_LOCKS=0` を設定する方針を検証する。ただし Git プロセスから全種類の書き込みが絶対起こらないと無検証で断言しない

**実測した注意事項（Git 2.47.3 / Linux）:**

- `check-ignore -z` をコマンドライン上のパス引数と組み合わせると `fatal: -z only makes sense with --stdin` になる。stdin入力が必要
- `git --literal-pathspecs ... check-ignore --stdin ...` は `pathspec magic not supported` を返した。**グローバルの literal pathspec 設定を全コマンドに一律適用しない**。このコマンドでは stdin の各パスを直接問い合わせる
- `git check-attr -z` は `path\0attribute\0value\0` を返す
- `git status --porcelain=v1 -z` はフィールドの順序・rename時の追加パスに注意する

## パスと信頼境界

- `..` を含む入力の lexical normalization だけで所属判定をしない。実在する対象の canonical path と Git が認識した worktree を照合する
- root 外を指す symlink / junction / reparse point は **MVPでは拒否**。安全性を証明できるまで参照先に沿って外へ出ない
- 選択パス自体が `.git` 以下なら拒否（Git内部を通常の追跡対象と見なさない）
- bare repository は対象外
- submodule は親から無条件に取り込まない。初期版では submodule の内外を別リポジトリとして識別できない場合は拒否
- 別 worktree の `.git` はファイルの場合がある。必ず `rev-parse` で実体を決める
- UNC、権限なし、壊れたリンク、削除直後の選択等は「不明」として適用を禁止。エラーと部分結果を返す
- フォルダー内ファイルの全走査や複数リポジトリへの一括操作は #12 に譲る。MVPで `mixed` を算出するために内部でファイル列挙を行わない

## 実装順とテストゲート

1. `ProcessRunner` に NUL区切り stdin を安全に渡せる機能（大量入力での stdout/stderr デッドロック防止も設計）
2. `RepositoryLocator` + `SelectionGuard`、G01/G02、特殊文字・Unicode のテスト
3. index/status parser、tracked/unmerged、NUL・renameのテスト
4. ignore/check-attr parser と根拠情報、I05のテスト
5. worktree/submodule/symlink/junction 境界、G03/G04/S04のテスト
6. `scan` で型付き結果を返し、CLIの `--help`/出力形式と互換性を確認

受け入れ条件: 実Gitでの判定結果一致、対象外書き込みゼロ、特殊文字名の正確な扱い、異なるリポジトリの分離。現時点でCI（Issue #3）がジョブ実行に到達しない環境上の失敗を示すため、実装は未検証コードをマージせず、CIが稼働してから完了判定する。

## 参考

- [git-ls-files](https://git-scm.com/docs/git-ls-files)
- [git-status](https://git-scm.com/docs/git-status)
- [git-check-ignore](https://git-scm.com/docs/git-check-ignore)
- [git-check-attr](https://git-scm.com/docs/git-check-attr)
- [git-rev-parse](https://git-scm.com/docs/git-rev-parse)
