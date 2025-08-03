# LazyDB

RustでTUI（Terminal User Interface）を使用して作成されたデータベースクライアント。lazygitやlazydockerのような操作感で、複数のデータベース接続を管理できます。

## 特徴

- 📊 複数のデータベースタイプをサポート（PostgreSQL、MySQL、SQLite、MongoDB）
- 🗂️ プロジェクトベースの接続管理
- ⌨️ lazygit/lazydockerライクなキーボード操作
- 🔧 設定ファイルによる接続情報の永続化

## ビルドと実行

```bash
# プロジェクトのビルド
cargo build

# アプリケーションの実行
cargo run
```

## キーバインド

- `q`: アプリケーションの終了
- `Tab`: ビュー間の切り替え
- `Esc`: 接続リストビューに戻る
- `Enter`: 項目の選択

## ビュー

1. **接続リストビュー**: プロジェクトと接続の一覧表示
2. **データベースエクスプローラー**: テーブル構造と内容の表示
3. **クエリエディター**: SQLクエリの実行

## 設定

設定ファイルは以下の場所に保存されます：
- macOS: `~/Library/Application Support/lazydb/config.json`
- Linux: `~/.config/lazydb/config.json`
- Windows: `%APPDATA%\lazydb\config.json`

## 開発ステータス

このプロジェクトは初期開発段階です。現在実装されている機能：

- ✅ 基本的なTUIインターフェース
- ✅ 設定ファイルの読み書き
- ✅ 基本的なビュー切り替え
- ⚠️ データベース接続（実装予定）
- ⚠️ クエリ実行（実装予定）
- ⚠️ データ表示（実装予定）

## 今後の予定

- PostgreSQL、MySQL、SQLite、MongoDBドライバの実装
- 実際のデータベース接続機能
- クエリ実行と結果表示
- 接続設定の追加・編集UI
- プロジェクト管理機能