use std::path::Path;

use crate::export::Encoding;
use crate::model::QueryResult;

pub fn export(_result: &QueryResult, _path: &Path, _encoding: Encoding) -> anyhow::Result<()> {
    todo!("CSV export not implemented yet")
}
