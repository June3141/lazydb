//! Database provider abstraction layer

#![allow(dead_code)]
#![allow(unused_imports)]

mod postgres;
mod provider;

pub use postgres::PostgresProvider;
pub use provider::{DatabaseProvider, DatabaseType, ProviderError};
