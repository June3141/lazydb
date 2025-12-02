# Architecture

lazydb は **The Elm Architecture (TEA)** を採用した TUI アプリケーションです。

## The Elm Architecture (TEA)

TEA は関数型プログラミングに由来するシンプルなアーキテクチャパターンで、以下の3つの要素で構成されます。

```text
┌─────────────────────────────────────────────────────────┐
│                      Event Loop                         │
│                                                         │
│   ┌─────────┐    ┌──────────┐    ┌─────────────────┐   │
│   │  View   │───▶│ Message  │───▶│     Update      │   │
│   │ (draw)  │    │ (Event)  │    │ (state change)  │   │
│   └─────────┘    └──────────┘    └─────────────────┘   │
│        ▲                                   │            │
│        │         ┌──────────┐              │            │
│        └─────────│  Model   │◀─────────────┘            │
│                  │ (state)  │                           │
│                  └──────────┘                           │
└─────────────────────────────────────────────────────────┘
```

### 構成要素

| 要素 | 説明 | lazydb での実装 |
|------|------|-----------------|
| **Model** | アプリケーションの状態 | `App` 構造体 (`src/app.rs`) |
| **Message** | ユーザー操作やイベント | `Message` enum (`src/message.rs`) |
| **Update** | Message を受けて Model を更新 | `App::update()` メソッド |
| **View** | Model を元に UI を描画 | `ui::draw()` 関数 (`src/ui/mod.rs`) |

### 採用理由

1. **シンプルさ**: 状態管理が一箇所に集約され、デバッグしやすい
2. **予測可能性**: 同じ Message は常に同じ状態変化を起こす
3. **テスト容易性**: Update 関数を純粋関数として単体テスト可能
4. **拡張性**: 新しい機能は新しい Message を追加するだけ

## ディレクトリ構成

```text
src/
├── main.rs           # エントリーポイント、イベントループ
├── app.rs            # Model: アプリケーション状態と Update ロジック
├── message.rs        # Message: イベント定義
├── model/            # データモデル
│   ├── mod.rs
│   ├── connection.rs # データベース接続情報
│   ├── table.rs      # テーブル・カラム情報
│   └── query.rs      # クエリ結果
└── ui/               # View: UI描画
    ├── mod.rs        # レイアウト構成、draw() 関数
    ├── sidebar.rs    # サイドバー (接続ツリー、テーブルサマリー)
    ├── main_panel.rs # メインパネル (Schema/Data タブ、クエリエディタ)
    ├── status_bar.rs # ステータスバー
    ├── help_bar.rs   # ヘルプバー
    └── utils.rs      # ユーティリティ関数
```

## データフロー

```text
1. ユーザーがキーを押す
       │
       ▼
2. main.rs でキーイベントを Message に変換
       │
       ▼
3. App::update(message) で状態を更新
       │
       ▼
4. ui::draw(&app) で画面を再描画
       │
       ▼
5. 1に戻る (ループ)
```

### 例: テーブル選択の流れ

```rust
// 1. ユーザーが Enter キーを押す
KeyCode::Enter => Message::Activate

// 2. Update: 選択状態を更新
fn update(&mut self, msg: Message) -> bool {
    match msg {
        Message::Activate => {
            self.activate_current_item();
            false
        }
        // ...
    }
}

// 3. View: 選択されたテーブルの情報を表示
fn draw_table_summary(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        // テーブル情報を描画
    }
}
```

## UI レイアウト

```text
┌─────────────────────┬────────────────────────────────────────┐
│     Connections     │              SQL Query                 │
│  ▼ Production       ├────────────────────────────────────────┤
│    ├─ users         │  Schema [s] │ Data [d]                 │
│    ├─ orders        │────────────────────────────────────────│
│    └─ products      │                                        │
│  ▶ Development      │  (Schema or Data content)              │
│                     │                                        │
├─────────────────────┤                                        │
│       Info          ├────────────────────────────────────────┤
│  users              │              Status                    │
│  5 columns          │  ✓ 100 rows │ 12ms │ Ready            │
│  1,234 rows         │                                        │
└─────────────────────┴────────────────────────────────────────┘
 q Quit  ↑/k Up  ↓/j Down  Tab Focus  Enter Select  e Expand ...
```

## 使用ライブラリ

| ライブラリ | バージョン | 用途 |
|-----------|-----------|------|
| [ratatui](https://github.com/ratatui-org/ratatui) | 0.29 | TUI フレームワーク |
| [crossterm](https://github.com/crossterm-rs/crossterm) | 0.28 | ターミナルバックエンド |
| [anyhow](https://github.com/dtolnay/anyhow) | 1.0 | エラーハンドリング |
| [itertools](https://github.com/rust-itertools/itertools) | 0.13 | イテレータ拡張 |

## 拡張ガイド

### 新しいキーバインドを追加する

1. `src/message.rs` に新しい Message を追加
2. `src/main.rs` でキーを Message にマッピング
3. `src/app.rs` の `update()` で処理を実装

### 新しい UI コンポーネントを追加する

1. `src/ui/` に新しいファイルを作成
2. `src/ui/mod.rs` でモジュールを登録・使用
3. 必要に応じて `draw()` からコンポーネントを呼び出し

### 新しいデータモデルを追加する

1. `src/model/` に新しいファイルを作成
2. `src/model/mod.rs` で pub use でエクスポート
3. `src/app.rs` で必要に応じてインポート・使用
