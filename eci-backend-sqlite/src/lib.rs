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

/*
fn create_lock_table(conn: &Pool<SqliteConnectionManager>) -> Result<(), rusqlite::Error> {
    conn.get().unwrap().execute_batch(
        "
        create table if not exists locks (
            lockid    TEXT NOT NULL,
            entity    TEXT NOT NULL,
            component TEXT NOT NULL,
            version   TEXT NOT NULL,
            locktype  TEXT NOT NULL,
            expires   TEXT NOT NULL
        ) STRICT;
    ",
    )
}
#[derive(Debug)]
struct Lock {
    conn: Pool<SqliteConnectionManager>,
    id: Uuid,
}

#[must_use = "if the lock isn't captured, it will be immediately released"]
impl Lock {
    pub fn release(&self) {
        self.conn
            .get()
            .unwrap()
            .execute(
                "
        delete from locks where lockid = :id;
        ",
                named_params! {
                    ":id": self.id.to_string()
                },
            )
            .unwrap();
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        self.release()
    }
}

fn acquire_lock<T: Component>(
    conn: &Pool<SqliteConnectionManager>,
    entity: Entity,
    mode: Mode,
) -> Result<Option<Lock>, rusqlite::Error> {
    #[derive(Debug, Serialize)]
    struct SqlLock {
        lockid: String,
        entity: String,
        component: String,
        version: String,
        expires: DateTime<Utc>,
    }

    let lockid = Uuid::new_v4();

    let inserted = {
        let conn = conn.get().unwrap();

        let mut stmt = conn.prepare(match mode {
            Mode::Read => {
                "
                insert into locks
                select
                    :lockid    as lockid,
                    :entity    as entity,
                    :component as component,
                    :version   as version,
                    'read'     as locktype,
                    :expires   as expires
                where not exists(
                    select entity from locks
                    where locktype = 'write'
                    and entity     = :entity
                    and component  = :component
                    and version    = :version
                    and datetime(current_timestamp) < datetime(expires)
                );
            "
            }
            Mode::Write => {
                "
            insert into locks
                select
                    :lockid    as lockid,
                    :entity    as entity,
                    :component as component,
                    :version   as version,
                    'write'    as locktype,
                    :expires   as expires
                where not exists(
                    select entity from locks
                    where entity   = :entity
                    and component  = :component
                    and version    = :version
                    and datetime(current_timestamp) < datetime(expires)
                );
            "
            }
        })?;

        let params = to_params_named(&SqlLock {
            lockid: lockid.to_string(),
            entity: entity.to_string(),
            component: T::NAME.to_string(),
            version: T::VERSION.to_string(),
            expires: Utc::now() + Duration::hours(1),
        })
        .unwrap();

        stmt.execute(params.to_slice().as_slice())?
    } > 0;

    if inserted {
        Ok(Some(Lock {
            conn: conn.clone(),
            id: lockid,
        }))
    } else {
        Ok(None)
    }
}

fn create_component_table<T: Component>(
    conn: &Pool<SqliteConnectionManager>,
) -> Result<(), rusqlite::Error> {
    let name = T::NAME;

    let table = format!(
        "
        create table if not exists {name}(
            entity    text not null unique,
            version   text not null,
            component text not null
        ) strict;
    "
    );
    conn.get().unwrap().execute_batch(&table)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("rusqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serde_json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("serde to rusqlite mapping error: {0}")]
    SerdeRusqlite(#[from] serde_rusqlite::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SqlRepresentation {
    entity: String,
    version: String,
    component: String,
}

fn insert_component<T: Component + Serialize>(
    conn: &Pool<SqliteConnectionManager>,
    entity: Entity,
    component: T,
) -> Result<usize, Error> {
    let representation = SqlRepresentation {
        entity: entity.0.to_string(),
        version: T::VERSION.to_string(),
        component: serde_json::to_string(&component).unwrap(),
    };

    let name = T::NAME;
    let stmt = format!(
        "INSERT INTO {name} (entity, version, component) VALUES(:entity, :version, :component)"
    );

    let params = to_params_named(&representation).unwrap();

    Ok(conn
        .get()
        .unwrap()
        .execute(&stmt, params.to_slice().as_slice())?)
}

fn read_component<T: Component + DeserializeOwned>(
    conn: &Pool<SqliteConnectionManager>,
    entity: Entity,
) -> Result<Option<T>, Error> {
    let name = T::NAME;
    let conn = conn.get().unwrap();
    let mut stmt = conn.prepare(&format!("SELECT * FROM {name} WHERE entity = ? LIMIT 1"))?;

    let results = from_rows::<SqlRepresentation>(stmt.query([entity.0.to_string()])?);

    if let Some(row) = results.last() {
        let representation = row?;

        assert_eq!(representation.version, T::VERSION.to_string());
        let component: T = serde_json::from_str(&representation.component)?;
        Ok(Some(component))
    } else {
        Ok(None)
    }
}

fn read_two_components<A: Component + DeserializeOwned, B: Component + DeserializeOwned>(
    conn: &Pool<SqliteConnectionManager>,
    entity: Entity,
) -> Result<Option<(A, B)>, Error> {
    let conn = conn.get().unwrap();
    let mut stmt = conn.prepare(&format!(
        "
        select
        DebugString.version as a_vers,
        DebugString.component as a_comp,
        OtherComponent.version as b_vers,
        OtherComponent.component as b_comp
        from DebugString
        inner join OtherComponent on OtherComponent.entity = DebugString.entity
        where DebugString.entity = ?
        limit 1
    "
    ))?;

    let result = stmt
        .query_map([entity.0.to_string()], |row| {
            Ok((
                SqlRepresentation {
                    entity: "".to_string(),
                    version: row.get(0)?,
                    component: row.get(1)?,
                },
                SqlRepresentation {
                    entity: "".to_string(),
                    version: row.get(2)?,
                    component: row.get(3)?,
                },
            ))
        })?
        .last();

    if let Some(result) = result {
        let (a, b) = result?;

        Ok(Some((
            serde_json::from_str(&a.component)?,
            serde_json::from_str(&b.component)?,
        )))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use eci_core::{component::DebugString, Component, Entity, Version};
    use r2d2_sqlite::SqliteConnectionManager;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct OtherComponent {
        pub inner: String,
    }

    impl Component for OtherComponent {
        const NAME: &'static str = "OtherComponent";
        const VERSION: Version = Version::new(1, 0, 0);
    }

    use crate::*;

    #[test]
    fn test_coexisting_read_locks() {
        let conn = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();
        create_lock_table(&conn).unwrap();

        let entity = Entity::new();

        acquire_lock::<DebugString>(&conn, entity, Mode::Read)
            .unwrap()
            .unwrap();
        acquire_lock::<DebugString>(&conn, entity, Mode::Read)
            .unwrap()
            .unwrap();
        acquire_lock::<DebugString>(&conn, entity, Mode::Read)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn test_exclusive_locks() {
        let conn = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();
        create_lock_table(&conn).unwrap();

        // It should not be possible to acquire a read-only lock if a write lock exists
        let entity = Entity::new();
        let _w = acquire_lock::<DebugString>(&conn, entity, Mode::Write)
            .unwrap()
            .unwrap();
        assert!(acquire_lock::<DebugString>(&conn, entity, Mode::Read)
            .unwrap()
            .is_none());

        // It should not be possible to acquire a write lock if a read-only lock exists
        let entity = Entity::new();
        let _w = acquire_lock::<DebugString>(&conn, entity, Mode::Read)
            .unwrap()
            .unwrap();
        assert!(acquire_lock::<DebugString>(&conn, entity, Mode::Write)
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_lock_releasing() {
        let conn = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();
        create_lock_table(&conn).unwrap();

        let entity = Entity::new();
        {
            let _write_lock = acquire_lock::<DebugString>(&conn, entity, Mode::Write)
                .unwrap()
                .unwrap();
            assert!(acquire_lock::<DebugString>(&conn, entity, Mode::Read)
                .unwrap()
                .is_none());
        }

        assert!(acquire_lock::<DebugString>(&conn, entity, Mode::Read)
            .unwrap()
            .is_some());
    }

    #[test]
    fn it_works() {
        let conn = r2d2::Pool::new(SqliteConnectionManager::memory()).unwrap();

        create_component_table::<DebugString>(&conn).unwrap();
        create_component_table::<OtherComponent>(&conn).unwrap();

        let teststr = DebugString {
            content: Some("lol".to_string()),
        };

        let entity = Entity::new();

        insert_component(&conn, entity, teststr).unwrap();
        insert_component(
            &conn,
            entity,
            OtherComponent {
                inner: "Hello world!".to_string(),
            },
        )
        .unwrap();

        let readstr1 = read_component::<DebugString>(&conn, entity).unwrap();
        let readstr2 = read_component::<OtherComponent>(&conn, entity).unwrap();
        println!("{:#?}, {:#?}", readstr1, readstr2);

        let (a, b) = read_two_components::<DebugString, OtherComponent>(&conn, entity)
            .unwrap()
            .unwrap();

        println!("{:?}, {:?}", a, b);
    }
}
 */
