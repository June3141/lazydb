//! PostgresProvider struct and connection methods

use postgres::{Client, NoTls};
use r2d2::PooledConnection;
use r2d2_postgres::PostgresConnectionManager;
use std::sync::Mutex;

use crate::config::ConnectionConfig;

use super::pool::{ConnectionPool, PoolState};
use super::ProviderError;

/// Connection source for PostgresProvider
enum ConnectionSource {
    /// Single connection wrapped in a Mutex (boxed to reduce enum size)
    Single(Box<Mutex<Client>>),
    /// Connection pool
    Pool(ConnectionPool),
}

/// PostgreSQL database provider
pub struct PostgresProvider {
    source: ConnectionSource,
}

/// A wrapper that provides a uniform interface for both single and pooled connections
pub(super) enum ConnectionGuard<'a> {
    Single(std::sync::MutexGuard<'a, Client>),
    /// Boxed to reduce enum size difference
    Pooled(Box<PooledConnection<PostgresConnectionManager<NoTls>>>),
}

impl<'a> std::ops::Deref for ConnectionGuard<'a> {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        match self {
            ConnectionGuard::Single(guard) => guard,
            ConnectionGuard::Pooled(conn) => conn,
        }
    }
}

impl<'a> std::ops::DerefMut for ConnectionGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ConnectionGuard::Single(guard) => &mut *guard,
            ConnectionGuard::Pooled(conn) => conn,
        }
    }
}

impl PostgresProvider {
    /// Create a new PostgresProvider from connection configuration
    pub fn new(config: &ConnectionConfig) -> Result<Self, ProviderError> {
        let password = config.get_password().unwrap_or_default();
        let username = config.username.as_deref().unwrap_or("postgres");

        let connection_string = format!(
            "host={} port={} dbname={} user={} password={}",
            config.host, config.port, config.database, username, password
        );

        let client = Client::connect(&connection_string, NoTls)
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            source: ConnectionSource::Single(Box::new(Mutex::new(client))),
        })
    }

    /// Create a new PostgresProvider from connection parameters.
    ///
    /// Uses the `postgres::Config` builder API to safely handle passwords
    /// containing special characters (like `@`, `#`, spaces, or quotes).
    pub fn connect(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, ProviderError> {
        let mut config = postgres::Config::new();
        config
            .host(host)
            .port(port)
            .dbname(database)
            .user(username)
            .password(password);

        let client = config
            .connect(NoTls)
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            source: ConnectionSource::Single(Box::new(Mutex::new(client))),
        })
    }

    /// Create a new PostgresProvider using a connection pool.
    ///
    /// This is the preferred method for applications that need to handle
    /// multiple concurrent database operations efficiently.
    pub fn with_pool(pool: ConnectionPool) -> Self {
        Self {
            source: ConnectionSource::Pool(pool),
        }
    }

    /// Create a new PostgresProvider with a connection pool using default pool settings.
    ///
    /// This is a convenience method that combines connection and pool creation
    /// in a single call. For custom pool configuration, use [`ConnectionPool::new`]
    /// followed by [`PostgresProvider::with_pool`].
    pub fn connect_with_pool(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, ProviderError> {
        let pool = ConnectionPool::with_defaults(host, port, database, username, password)?;
        Ok(Self::with_pool(pool))
    }

    /// Get a connection from the provider.
    ///
    /// For single-connection mode, this acquires the mutex lock.
    /// For pooled mode, this gets a connection from the pool.
    pub(super) fn get_connection(&self) -> Result<ConnectionGuard<'_>, ProviderError> {
        match &self.source {
            ConnectionSource::Single(mutex) => {
                let guard = mutex.lock().map_err(|e| {
                    ProviderError::InternalError(format!("Failed to acquire client lock: {}", e))
                })?;
                Ok(ConnectionGuard::Single(guard))
            }
            ConnectionSource::Pool(pool) => {
                let conn = pool.get()?;
                Ok(ConnectionGuard::Pooled(Box::new(conn)))
            }
        }
    }

    /// Get the pool state if using a connection pool.
    ///
    /// Returns `None` if using a single connection.
    pub fn pool_state(&self) -> Option<PoolState> {
        match &self.source {
            ConnectionSource::Single(_) => None,
            ConnectionSource::Pool(pool) => Some(pool.state()),
        }
    }

    /// Check if this provider is using a connection pool.
    pub fn is_pooled(&self) -> bool {
        matches!(&self.source, ConnectionSource::Pool(_))
    }
}
