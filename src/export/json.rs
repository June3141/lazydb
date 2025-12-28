use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::export::Encoding;
use crate::model::QueryResult;

/// 文字列をエンコードしてバイト列を返す
fn encode_string(content: &str, encoding: Encoding) -> Vec<u8> {
    match encoding {
        Encoding::Utf8 => content.as_bytes().to_vec(),
        Encoding::ShiftJis => {
            let (encoded, _, _) = encoding_rs::SHIFT_JIS.encode(content);
            encoded.into_owned()
        }
        Encoding::EucJp => {
            let (encoded, _, _) = encoding_rs::EUC_JP.encode(content);
            encoded.into_owned()
        }
    }
}

pub fn export(result: &QueryResult, path: &Path, encoding: Encoding) -> anyhow::Result<()> {
    // 各行をオブジェクトに変換
    let rows: Vec<serde_json::Map<String, serde_json::Value>> = result
        .rows
        .iter()
        .map(|row| {
            let mut obj = serde_json::Map::new();
            for (i, col) in result.columns.iter().enumerate() {
                let value = row.get(i).cloned().unwrap_or_default();
                obj.insert(col.clone(), serde_json::Value::String(value));
            }
            obj
        })
        .collect();

    // JSONにシリアライズ（整形出力）
    let json = serde_json::to_string_pretty(&rows)?;

    // エンコードしてファイルに書き込み
    let bytes = encode_string(&json, encoding);
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;

    Ok(())
}
