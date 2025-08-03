# LazyDB 設定とセキュリティ

## 概要

LazyDBでは、データベース接続設定をYAMLファイルで管理します。パスワードは暗号化され、設定ファイルの肥大化を防ぐ工夫も施されています。

## 設定ファイルの場所

設定ファイルは `~/.config/lazydb/config.yaml` に保存されます。

## セキュリティ機能

### パスワードの暗号化

パスワードは以下の方法で安全に保存されます：

1. **PBKDF2-HMAC-SHA256**を使用してパスワードをハッシュ化
2. **10万回の反復処理**でレインボーテーブル攻撃を防止
3. **ランダムソルト**で同じパスワードでも異なるハッシュを生成
4. **Base64エンコード**でYAMLに安全に保存

```yaml
secure_password:
  salt: "ランダムに生成されたソルト（Base64）"
  hash: "PBKDF2で生成されたハッシュ（Base64）"
```

### セキュリティの利点

- **プレーンテキストなし**: パスワードは生データで保存されません
- **リバースエンジニアリング困難**: ハッシュからパスワードを復元不可能
- **ソルト使用**: 同じパスワードでも異なるハッシュになります
- **高コスト**: 10万回の反復でブルートフォース攻撃を困難にします

## 設定ファイルの構造

### 基本的な接続設定

```yaml
connections:
  - id: "unique_connection_id"
    name: "表示名"
    database_type: "PostgreSQL"  # PostgreSQL, MySQL, SQLite, MongoDB
    host: "localhost"
    port: 5432
    username: "database_user"
    database_name: "database_name"
    # パスワードは実行時に設定され、暗号化されて保存されます
    tags:
      - "production"
      - "primary"
```

### 肥大化防止の工夫

#### 1. Connection Groups

関連する接続をグループ化して整理：

```yaml
connection_groups:
  - name: "Production"
    description: "本番環境のデータベース"
    connections:
      - "postgres_prod"
      - "mysql_prod"
  
  - name: "Development"
    description: "開発環境のデータベース"
    connections:
      - "postgres_dev"
      - "sqlite_local"
```

#### 2. Tags

接続にタグを付けて分類：

```yaml
connections:
  - id: "api_db"
    name: "API Database"
    # ...
    tags:
      - "api"
      - "read-only"
      - "cached"
```

#### 3. Projects

複数の接続をプロジェクト単位で管理：

```yaml
projects:
  - id: "web_app"
    name: "Web Application"
    connection_ids:
      - "postgres_prod"
      - "redis_cache"
      - "analytics_db"
```

#### 4. オプショナルフィールド

不要なフィールドは自動的に省略されます：

- `secure_password`: パスワードが設定されていない場合
- `database_name`: データベース名が不要な場合
- `tags`: タグが設定されていない場合
- `description`: 説明が設定されていない場合

## 使用例

### 1. 基本的な使用方法

```rust
use lazydb::config::{Config, Connection, DatabaseType};

// 設定をロード
let mut config = Config::load()?;

// 新しい接続を作成
let mut connection = Connection::new(
    "my_db".to_string(),
    "My Database".to_string(),
    DatabaseType::PostgreSQL,
    "localhost".to_string(),
    5432,
    "user".to_string(),
    Some("mydb".to_string()),
);

// パスワードを安全に設定
connection.set_password("my_secret_password")?;

// 設定に追加
config.add_connection(connection);

// 設定を保存
config.save()?;
```

### 2. パスワード検証

```rust
// パスワードの確認
let is_valid = connection.verify_password("my_secret_password")?;
assert!(is_valid);

// 間違ったパスワード
let is_invalid = connection.verify_password("wrong_password")?;
assert!(!is_invalid);
```

### 3. 設定の検証

```rust
// 設定の整合性をチェック
config.validate()?;
```

## 設定ファイルの例

完全な設定例は `config_example.yaml` を参照してください。

## セキュリティのベストプラクティス

1. **定期的なパスワード変更**: セキュアハッシュを使用していても定期更新を推奨
2. **ファイル権限**: 設定ファイルのアクセス権限を適切に設定（`chmod 600`）
3. **バックアップ**: 設定ファイルのセキュアなバックアップを作成
4. **バージョン管理除外**: `.gitignore`に設定ファイルを追加

## トラブルシューティング

### YAML解析エラー

```bash
# 設定ファイルの構文チェック
yamllint ~/.config/lazydb/config.yaml
```

### パスワード検証失敗

パスワードの検証に失敗する場合：
1. パスワードを再設定
2. 設定ファイルを再生成
3. キャッシュをクリア

### ファイル権限エラー

```bash
# 設定ディレクトリの権限を修正
chmod 700 ~/.config/lazydb
chmod 600 ~/.config/lazydb/config.yaml
```