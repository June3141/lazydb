#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub row_count: usize,
    pub columns: Vec<Column>,
    pub size_bytes: u64,
}
