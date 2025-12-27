use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::export::Encoding;
use crate::model::QueryResult;

/// CSVの値をエスケープする
/// カンマ、ダブルクォート、改行を含む場合はダブルクォートで囲む
fn escape_csv_value(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        // ダブルクォートを二重にしてエスケープ
        let escaped = value.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        value.to_string()
    }
}

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
    let mut output = String::new();

    // ヘッダー行
    let header: Vec<String> = result.columns.iter().map(|c| escape_csv_value(c)).collect();
    output.push_str(&header.join(","));
    output.push('\n');

    // データ行
    for row in &result.rows {
        let escaped_row: Vec<String> = row.iter().map(|v| escape_csv_value(v)).collect();
        output.push_str(&escaped_row.join(","));
        output.push('\n');
    }

    // エンコードしてファイルに書き込み
    let bytes = encode_string(&output, encoding);
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_value_simple() {
        assert_eq!(escape_csv_value("hello"), "hello");
    }

    #[test]
    fn test_escape_csv_value_with_comma() {
        assert_eq!(escape_csv_value("hello, world"), "\"hello, world\"");
    }

    #[test]
    fn test_escape_csv_value_with_quotes() {
        assert_eq!(escape_csv_value("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_escape_csv_value_with_newline() {
        assert_eq!(escape_csv_value("line1\nline2"), "\"line1\nline2\"");
    }
}
