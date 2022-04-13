mod access;
mod lock;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub use lock::SqliteLock;

pub struct SqliteBackend(Pool<SqliteConnectionManager>);

impl TryFrom<Pool<SqliteConnectionManager>> for SqliteBackend {
    type Error = rusqlite::Error;
    fn try_from(pool: Pool<SqliteConnectionManager>) -> Result<Self, Self::Error> {
        lock::create_lock_table(&pool)?;
        Ok(SqliteBackend(pool))
    }
}

impl SqliteBackend {
    pub fn in_memory() -> Result<Self, r2d2::Error> {
        let pool = r2d2::Pool::new(SqliteConnectionManager::memory())?;

        lock::create_lock_table(&pool).unwrap();
        Ok(SqliteBackend(pool))
    }
}
