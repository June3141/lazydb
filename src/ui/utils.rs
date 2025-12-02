use itertools::Itertools;

/// Format bytes to human readable size
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format number with thousand separators
pub fn format_number(n: usize) -> String {
    n.to_string()
        .chars()
        .rev()
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect::<String>())
        .join(",")
        .chars()
        .rev()
        .collect()
}
