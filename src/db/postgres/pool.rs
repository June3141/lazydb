//! Connection pool for PostgreSQL
//!
//! Provides connection pooling using r2d2 for improved performance
//! by reusing database connections instead of creating new ones for each operation.

use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use std::time::Duration;

use super::ProviderError;

/// Default maximum number of connections in the pool
pub const DEFAULT_MAX_SIZE: u32 = 10;
/// Default minimum number of idle connections to maintain
pub const DEFAULT_MIN_IDLE: u32 = 1;
/// Default connection timeout in seconds
pub const DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 30;
/// Default maximum lifetime of a connection in seconds (30 minutes)
pub const DEFAULT_MAX_LIFETIME_SECS: u64 = 30 * 60;
/// Default idle timeout in seconds (10 minutes)
pub const DEFAULT_IDLE_TIMEOUT_SECS: u64 = 10 * 60;

/// Configuration for the connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_size: u32,
    /// Minimum number of idle connections to maintain
    pub min_idle: Option<u32>,
    /// Maximum time to wait for a connection from the pool
    pub connection_timeout: Duration,
    /// Maximum lifetime of a connection
    pub max_lifetime: Option<Duration>,
    /// Time after which idle connections are closed
    pub idle_timeout: Option<Duration>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: DEFAULT_MAX_SIZE,
            min_idle: Some(DEFAULT_MIN_IDLE),
            connection_timeout: Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECS),
            max_lifetime: Some(Duration::from_secs(DEFAULT_MAX_LIFETIME_SECS)),
            idle_timeout: Some(Duration::from_secs(DEFAULT_IDLE_TIMEOUT_SECS)),
        }
    }
}

impl PoolConfig {
    /// Validate the configuration values
    ///
    /// Returns an error if:
    /// - `max_size` is 0
    /// - `min_idle` is greater than `max_size`
    pub fn validate(&self) -> Result<(), ProviderError> {
        if self.max_size == 0 {
            return Err(ProviderError::InvalidConfiguration(
                "max_size must be greater than 0".to_string(),
            ));
        }
        if let Some(min_idle) = self.min_idle {
            if min_idle > self.max_size {
                return Err(ProviderError::InvalidConfiguration(format!(
                    "min_idle ({}) cannot be greater than max_size ({})",
                    min_idle, self.max_size
                )));
            }
        }
        Ok(())
    }
}

/// A connection pool for PostgreSQL connections.
///
/// # Thread Safety
///
/// `ConnectionPool` is both `Send` and `Sync`, making it safe to share across threads.
/// The underlying r2d2 pool handles connection management and synchronization internally.
/// Multiple threads can safely call `get()` concurrently to obtain connections.
#[derive(Clone)]
pub struct ConnectionPool {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl ConnectionPool {
    /// Create a new connection pool with the given configuration
    pub fn new(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
        config: PoolConfig,
    ) -> Result<Self, ProviderError> {
        // Validate configuration before creating pool
        config.validate()?;

        let mut pg_config = postgres::Config::new();
        pg_config
            .host(host)
            .port(port)
            .dbname(database)
            .user(username)
            .password(password);

        let manager = PostgresConnectionManager::new(pg_config, NoTls);

        let pool = Pool::builder()
            .max_size(config.max_size)
            .min_idle(config.min_idle)
            .connection_timeout(config.connection_timeout)
            .max_lifetime(config.max_lifetime)
            .idle_timeout(config.idle_timeout)
            .build(manager)
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Create a new connection pool with default configuration
    pub fn with_defaults(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, ProviderError> {
        Self::new(
            host,
            port,
            database,
            username,
            password,
            PoolConfig::default(),
        )
    }

    /// Get a connection from the pool
    pub fn get(&self) -> Result<PooledConnection<PostgresConnectionManager<NoTls>>, ProviderError> {
        self.pool.get().map_err(|e| {
            let state = self.pool.state();
            ProviderError::ConnectionFailed(format!(
                "Failed to get connection from pool: {} (pool state: {} connections, {} idle, max {})",
                e, state.connections, state.idle_connections, self.pool.max_size()
            ))
        })
    }

    /// Get the current state of the pool
    pub fn state(&self) -> PoolState {
        let state = self.pool.state();
        PoolState {
            connections: state.connections,
            idle_connections: state.idle_connections,
        }
    }

    /// Get the maximum pool size
    pub fn max_size(&self) -> u32 {
        self.pool.max_size()
    }
}

/// State of the connection pool
#[derive(Debug, Clone, Copy)]
pub struct PoolState {
    /// Total number of connections in the pool
    pub connections: u32,
    /// Number of idle connections
    pub idle_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_size, DEFAULT_MAX_SIZE);
        assert_eq!(config.min_idle, Some(DEFAULT_MIN_IDLE));
        assert_eq!(
            config.connection_timeout,
            Duration::from_secs(DEFAULT_CONNECTION_TIMEOUT_SECS)
        );
    }

    #[test]
    fn test_pool_config_validation_success() {
        let config = PoolConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_pool_config_validation_max_size_zero() {
        let config = PoolConfig {
            max_size: 0,
            ..Default::default()
        };
        match config.validate() {
            Err(ProviderError::InvalidConfiguration(msg)) => {
                assert!(msg.contains("max_size"));
            }
            other => panic!("Expected InvalidConfiguration, got: {:?}", other),
        }
    }

    #[test]
    fn test_pool_config_validation_min_idle_greater_than_max() {
        let config = PoolConfig {
            max_size: 5,
            min_idle: Some(10),
            ..Default::default()
        };
        match config.validate() {
            Err(ProviderError::InvalidConfiguration(msg)) => {
                assert!(msg.contains("min_idle"));
                assert!(msg.contains("max_size"));
            }
            other => panic!("Expected InvalidConfiguration, got: {:?}", other),
        }
    }

    #[test]
    fn test_pool_creation_failure_invalid_host() {
        let result = ConnectionPool::with_defaults(
            "nonexistent.invalid.host.example.com",
            5432,
            "testdb",
            "user",
            "pass",
        );

        assert!(result.is_err());
        match result {
            Err(ProviderError::ConnectionFailed(msg)) => {
                println!("Expected connection failure: {}", msg);
            }
            Err(other) => panic!("Expected ConnectionFailed, got: {:?}", other),
            Ok(_) => panic!("Expected connection to fail"),
        }
    }

    #[test]
    fn test_pool_creation_failure_invalid_port() {
        let result = ConnectionPool::with_defaults("localhost", 1, "testdb", "user", "pass");

        assert!(result.is_err());
        match result {
            Err(ProviderError::ConnectionFailed(msg)) => {
                println!("Expected connection failure: {}", msg);
            }
            Err(other) => panic!("Expected ConnectionFailed, got: {:?}", other),
            Ok(_) => panic!("Expected connection to fail"),
        }
    }

    #[test]
    #[ignore] // Requires database connection
    fn test_pool_creation_success() {
        let pool =
            ConnectionPool::with_defaults("localhost", 5432, "lazydb_dev", "lazydb", "lazydb")
                .expect("Failed to create pool");

        let state = pool.state();
        assert!(
            state.connections >= 1,
            "Pool should have at least one connection"
        );
        assert_eq!(pool.max_size(), 10);
    }

    #[test]
    #[ignore] // Requires database connection
    fn test_pool_get_connection() {
        let pool =
            ConnectionPool::with_defaults("localhost", 5432, "lazydb_dev", "lazydb", "lazydb")
                .expect("Failed to create pool");

        // Get a connection and use it
        let mut conn = pool.get().expect("Failed to get connection");
        let result = conn
            .query_one("SELECT 1 as value", &[])
            .expect("Query failed");
        let value: i32 = result.get("value");
        assert_eq!(value, 1);
    }

    #[test]
    #[ignore] // Requires database connection
    fn test_pool_connection_reuse() {
        let pool =
            ConnectionPool::with_defaults("localhost", 5432, "lazydb_dev", "lazydb", "lazydb")
                .expect("Failed to create pool");

        // Get and release multiple connections
        for i in 0..5 {
            let mut conn = pool.get().expect("Failed to get connection");
            let result = conn
                .query_one("SELECT $1::int as value", &[&i])
                .expect("Query failed");
            let value: i32 = result.get("value");
            assert_eq!(value, i);
            // Connection is automatically returned to pool when dropped
        }

        let state = pool.state();
        // After releasing all connections, we should have idle connections
        assert!(
            state.idle_connections >= 1,
            "Pool should have idle connections after use"
        );
    }

    #[test]
    #[ignore] // Requires database connection
    fn test_pool_concurrent_connections() {
        use std::sync::Arc;
        use std::thread;

        let pool = Arc::new(
            ConnectionPool::new(
                "localhost",
                5432,
                "lazydb_dev",
                "lazydb",
                "lazydb",
                PoolConfig {
                    max_size: 5,
                    min_idle: Some(2),
                    ..Default::default()
                },
            )
            .expect("Failed to create pool"),
        );

        // Spawn multiple threads that use connections concurrently
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let pool_clone = Arc::clone(&pool);
                thread::spawn(move || {
                    let mut conn = pool_clone.get().expect("Failed to get connection");
                    let result = conn
                        .query_one("SELECT $1::int as value", &[&i])
                        .expect("Query failed");
                    let value: i32 = result.get("value");
                    assert_eq!(value, i);
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }

    #[test]
    #[ignore] // Requires database connection
    fn test_pool_custom_config() {
        let config = PoolConfig {
            max_size: 5,
            min_idle: Some(2),
            connection_timeout: Duration::from_secs(10),
            max_lifetime: Some(Duration::from_secs(60)),
            idle_timeout: Some(Duration::from_secs(30)),
        };

        let pool = ConnectionPool::new("localhost", 5432, "lazydb_dev", "lazydb", "lazydb", config)
            .expect("Failed to create pool");

        assert_eq!(pool.max_size(), 5);
    }
}
