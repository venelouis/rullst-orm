#[cfg(not(any(
    feature = "strict-postgres",
    feature = "strict-mysql",
    feature = "strict-sqlite"
)))]
pub use sqlx::AnyPool as RullstPool;

#[cfg(feature = "strict-postgres")]
pub use sqlx::PgPool as RullstPool;

#[cfg(all(feature = "strict-mysql", not(feature = "strict-postgres")))]
pub use sqlx::MySqlPool as RullstPool;

#[cfg(all(
    feature = "strict-sqlite",
    not(feature = "strict-postgres"),
    not(feature = "strict-mysql")
))]
pub use sqlx::SqlitePool as RullstPool;

#[cfg(not(any(
    feature = "strict-postgres",
    feature = "strict-mysql",
    feature = "strict-sqlite"
)))]
use sqlx::any::install_default_drivers;

use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

// Re-export the procedural macro so users only need to import `rullst-orm`
pub use futures;
pub use serde;
pub use serde_json;
pub use sqlx;

#[cfg(feature = "redis")]
pub use redis;
pub mod collection;
pub mod database;
pub mod audit;
pub mod resource;
pub mod scout;
pub mod admin;
pub mod schema;
pub mod tenant;
pub mod types;

// Re-exports
pub use collection::RullstCollection;
pub use database::RullstDatabase;
pub use rullst_orm_macros::Orm;
pub use resource::{ApiResource, JsonResource, ResourceCollection};
pub use scout::{get_search_engine, set_search_engine, SearchEngine};
pub use admin::dashboard_html;
pub use tenant::{get_tenant_id, with_tenant};
pub use types::Json;

// Re-export async_trait so the macro can use it implicitly
pub use async_trait::async_trait;

// Re-export sqlx and FromRow for database mapping
pub use schema::{JoinClause, SubqueryBuilder};
pub use sqlx::FromRow;

/// The global connection pool
static DB_POOL: OnceLock<RullstPool> = OnceLock::new();

/// The driver identifier (postgres, mysql, sqlite) to help macro syntax formatting
static DB_DRIVER: OnceLock<String> = OnceLock::new();

/// The replica connection pools for read operations
static REPLICA_POOLS: OnceLock<Vec<RullstPool>> = OnceLock::new();

/// Atomic index for replica round-robin selection
static REPLICA_INDEX: AtomicUsize = AtomicUsize::new(0);

#[cfg(feature = "redis")]
static REDIS_CLIENT: OnceLock<redis::Client> = OnceLock::new();

#[cfg(feature = "redis")]
static REDIS_MANAGER: OnceLock<redis::aio::ConnectionManager> = OnceLock::new();

/// Enum dinâmico para encapsular qualquer tipo que possa ser associado ao banco de dados pelo Macro
#[derive(Clone, Debug)]
pub enum RullstValue {
    String(String),
    Int(i32),
    Float(f64),
    Bool(bool),
}

impl From<&str> for RullstValue {
    fn from(s: &str) -> Self {
        RullstValue::String(s.to_string())
    }
}
impl From<String> for RullstValue {
    fn from(s: String) -> Self {
        RullstValue::String(s)
    }
}
impl From<i32> for RullstValue {
    fn from(i: i32) -> Self {
        RullstValue::Int(i)
    }
}
impl From<f64> for RullstValue {
    fn from(f: f64) -> Self {
        RullstValue::Float(f)
    }
}
impl From<bool> for RullstValue {
    fn from(b: bool) -> Self {
        RullstValue::Bool(b)
    }
}

impl TryFrom<RullstValue> for String {
    type Error = &'static str;
    fn try_from(val: RullstValue) -> Result<Self, Self::Error> {
        match val {
            RullstValue::String(s) => Ok(s),
            _ => Err("Not a string"),
        }
    }
}
impl TryFrom<RullstValue> for i32 {
    type Error = &'static str;
    fn try_from(val: RullstValue) -> Result<Self, Self::Error> {
        match val {
            RullstValue::Int(i) => Ok(i),
            _ => Err("Not an i32"),
        }
    }
}
impl TryFrom<RullstValue> for f64 {
    type Error = &'static str;
    fn try_from(val: RullstValue) -> Result<Self, Self::Error> {
        match val {
            RullstValue::Float(f) => Ok(f),
            _ => Err("Not an f64"),
        }
    }
}
impl TryFrom<RullstValue> for bool {
    type Error = &'static str;
    fn try_from(val: RullstValue) -> Result<Self, Self::Error> {
        match val {
            RullstValue::Bool(b) => Ok(b),
            _ => Err("Not a bool"),
        }
    }
}

/// Orm configuration structure
pub struct Orm;

impl Orm {
    /// Initialize the global database connection pool using an agnostic URI
    pub async fn init(database_url: &str) -> Result<(), sqlx::Error> {
        #[cfg(not(any(
            feature = "strict-postgres",
            feature = "strict-mysql",
            feature = "strict-sqlite"
        )))]
        install_default_drivers();

        let pool = RullstPool::connect(database_url).await?;

        if DB_POOL.set(pool).is_err() {
            panic!("Orm has already been initialized");
        }

        let driver = if database_url.starts_with("postgres") {
            "postgres"
        } else if database_url.starts_with("mysql") {
            "mysql"
        } else {
            "sqlite"
        };

        let _ = DB_DRIVER.set(driver.to_string());
        let _ = REPLICA_POOLS.set(vec![]);

        Ok(())
    }

    /// Initialize the global database connection pool and its read replicas
    pub async fn init_with_replicas(
        primary_url: &str,
        replica_urls: Vec<&str>,
    ) -> Result<(), sqlx::Error> {
        #[cfg(not(any(
            feature = "strict-postgres",
            feature = "strict-mysql",
            feature = "strict-sqlite"
        )))]
        install_default_drivers();

        let pool = RullstPool::connect(primary_url).await?;

        if DB_POOL.set(pool).is_err() {
            panic!("Orm has already been initialized");
        }

        let driver = if primary_url.starts_with("postgres") {
            "postgres"
        } else if primary_url.starts_with("mysql") {
            "mysql"
        } else {
            "sqlite"
        };

        let _ = DB_DRIVER.set(driver.to_string());

        let mut replicas = vec![];
        for url in replica_urls {
            let p = RullstPool::connect(url).await?;
            replicas.push(p);
        }
        let _ = REPLICA_POOLS.set(replicas);

        Ok(())
    }

    /// Retrieve the global database connection pool (strictly for writes)
    pub fn pool() -> &'static RullstPool {
        DB_POOL
            .get()
            .expect("Orm must be initialized before querying")
    }

    /// Retrieve the connection pool for read operations.
    /// Performs a round-robin load balancing over replicas if configured.
    pub fn read_pool() -> &'static RullstPool {
        if let Some(replicas) = REPLICA_POOLS.get()
            && !replicas.is_empty()
        {
            let idx = REPLICA_INDEX.fetch_add(1, Ordering::Relaxed) % replicas.len();
            return &replicas[idx];
        }
        Self::pool()
    }

    /// Retrieve the active driver string
    pub fn driver() -> &'static str {
        DB_DRIVER
            .get()
            .expect("Orm must be initialized before querying")
            .as_str()
    }

    /// Starts a new database transaction
    #[cfg(not(any(
        feature = "strict-postgres",
        feature = "strict-mysql",
        feature = "strict-sqlite"
    )))]
    pub async fn begin_transaction() -> Result<sqlx::Transaction<'static, sqlx::Any>, sqlx::Error> {
        let pool = Self::pool();
        pool.begin().await
    }

    #[cfg(feature = "strict-postgres")]
    pub async fn begin_transaction()
    -> Result<sqlx::Transaction<'static, sqlx::Postgres>, sqlx::Error> {
        let pool = Self::pool();
        pool.begin().await
    }

    #[cfg(all(feature = "strict-mysql", not(feature = "strict-postgres")))]
    pub async fn begin_transaction() -> Result<sqlx::Transaction<'static, sqlx::MySql>, sqlx::Error>
    {
        let pool = Self::pool();
        pool.begin().await
    }

    #[cfg(all(
        feature = "strict-sqlite",
        not(feature = "strict-postgres"),
        not(feature = "strict-mysql")
    ))]
    pub async fn begin_transaction() -> Result<sqlx::Transaction<'static, sqlx::Sqlite>, sqlx::Error>
    {
        let pool = Self::pool();
        pool.begin().await
    }

    /// Run an array of seeders sequentially
    pub async fn seed(seeders: Vec<Box<dyn Seeder>>) -> Result<(), sqlx::Error> {
        for seeder in seeders {
            seeder.run().await?;
        }
        Ok(())
    }

    /// Enable query logging to print all queries to the terminal
    pub fn enable_query_log() {
        crate::schema::enable_query_log();
    }

    /// Disable query logging
    pub fn disable_query_log() {
        crate::schema::disable_query_log();
    }

    /// Initialize Redis connection and connection manager for caching and events
    #[cfg(feature = "redis")]
    pub async fn init_redis(redis_url: &str) -> Result<(), redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        let manager = redis::aio::ConnectionManager::new(client.clone()).await?;
        let _ = REDIS_CLIENT.set(client);
        let _ = REDIS_MANAGER.set(manager);
        Ok(())
    }

    /// Get reference to the global Redis client
    #[cfg(feature = "redis")]
    pub fn redis_client() -> &'static redis::Client {
        REDIS_CLIENT
            .get()
            .expect("Redis must be initialized before using cache features")
    }

    /// Get clone of the thread-safe connection manager for async Redis queries
    #[cfg(feature = "redis")]
    pub fn redis_manager() -> redis::aio::ConnectionManager {
        REDIS_MANAGER
            .get()
            .expect("Redis must be initialized before using cache features")
            .clone()
    }
}

/// A database seeder trait for populating tables
#[async_trait]
pub trait Seeder: Send + Sync {
    async fn run(&self) -> Result<(), sqlx::Error>;
}

/// The core trait that all Orm models will implement via #[derive(Orm)]
#[async_trait]
pub trait RullstModel {
    fn table_name() -> &'static str;
}

/// Represents a paginated result set
#[derive(Debug, Clone)]
pub struct PaginationResult<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub per_page: usize,
    pub current_page: usize,
    pub last_page: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rullst_value_conversions() {
        let v: RullstValue = "test".into();
        assert!(matches!(v, RullstValue::String(_)));
        let v_int: RullstValue = 100.into();
        assert!(matches!(v_int, RullstValue::Int(100)));
        let v_bool: RullstValue = false.into();
        assert!(matches!(v_bool, RullstValue::Bool(false)));
    }
}
