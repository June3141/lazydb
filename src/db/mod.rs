//! Database provider abstraction layer

#![allow(dead_code)]
#![allow(unused_imports)]

mod async_bridge;
mod postgres;
mod provider;
mod worker;

pub use async_bridge::{ConnectionParams, DbCommand, DbResponse};
pub use postgres::PostgresProvider;
pub use provider::{DatabaseProvider, DatabaseType, ProviderError};
pub use worker::{spawn_db_worker, DbWorkerHandle};
