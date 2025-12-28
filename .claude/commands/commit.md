# git commit 作成コマンド

## 変更のコミット

- 基本的に全ファイルの変更をコミットする
- `git status` で変更ファイルを確認する
- commit の粒度は小さく、意味のある単位で行う
- 1 つの commit で変更される量は最大で 100 行以下を目安とする（ただし、新規モジュールや大きな機能追加など、分割が現実的でない場合は例外とする）
- commit は prefix と github emoji をつける
- commit は `prefix: emoji 説明文` の形式にする
- 使用する emoji は `.claude/commands/git_emoji.json` を参照する
- 英語で commit メッセージを書く
- main に commit は絶対に行わない
