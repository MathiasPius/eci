mod access;
mod lock;
use std::path::Path;

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
    pub fn memory() -> Result<Self, r2d2::Error> {
        let pool = r2d2::Pool::new(SqliteConnectionManager::memory())?;

        lock::create_lock_table(&pool).unwrap();
        Ok(SqliteBackend(pool))
    }

    pub fn file<P: AsRef<Path>>(path: P) -> Result<Self, r2d2::Error> {
        let pool = r2d2::Pool::new(SqliteConnectionManager::file(path))?;

        lock::create_lock_table(&pool).unwrap();
        Ok(SqliteBackend(pool))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dotest() {
        struct Test(i32);

        impl Test {
            fn get(&mut self) -> &mut i32 {
                &mut self.0
            }
        }

        let mut test = Test(10);
        let lol = test.get();
    }
}
