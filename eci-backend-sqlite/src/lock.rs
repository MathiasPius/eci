use chrono::{Duration, Utc};
use eci_core::backend::{Lock, LockDescriptor, LockingBackend, LockingError, LockingMode};
use log::*;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::named_params;
use uuid::Uuid;

use crate::SqliteBackend;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SqliteLock(Uuid);

const WRITE_LOCK: &'static str = "insert into locks
select
    :lockid    as lockid,
    :entity    as entity,
    :component as component,
    'write'    as locktype,
    :expires   as expires
where not exists(
    select entity from locks
    where entity   = :entity
    and component  = :component
    and datetime(current_timestamp) < datetime(expires)
);";

const READ_LOCK: &'static str = "insert into locks
select
    :lockid    as lockid,
    :entity    as entity,
    :component as component,
    'read'     as locktype,
    :expires   as expires
where not exists(
    select entity from locks
    where locktype = 'write'
    and entity     = :entity
    and component  = :component
    and datetime(current_timestamp) < datetime(expires)
);";

impl LockingBackend for SqliteBackend {
    fn acquire_lock(
        &self,
        entity: eci_core::Entity,
        descriptors: Vec<LockDescriptor>,
        expires_in: std::time::Duration,
    ) -> Result<Lock, LockingError> {
        let lock = Lock::new();

        let mut conn = self.0.get().map_err(LockingError::implementation)?;

        debug!("starting lock transaction for lock {lock}");
        let tx = conn.transaction().map_err(LockingError::implementation)?;

        for descriptor in descriptors {
            let params = named_params! {
                ":lockid": lock.id(),
                ":entity": entity.to_string(),
                ":component": descriptor.name,
                ":expires": Utc::now() + Duration::from_std(expires_in).map_err(LockingError::implementation)?,
            };

            debug!("acquiring {}-lock for {}", descriptor.mode, descriptor.name);

            if tx
                .execute(
                    match descriptor.mode {
                        LockingMode::Read => READ_LOCK,
                        LockingMode::Write => WRITE_LOCK,
                    },
                    params,
                )
                .map_err(LockingError::implementation)?
                != 1
            {
                return Err(LockingError::Conflict(
                    entity,
                    descriptor.name,
                    descriptor.mode,
                ));
            };
        }

        tx.commit().map_err(LockingError::implementation)?;
        debug!("lock {lock} transaction committed");

        Ok(lock)
    }

    fn release_lock(&self, lock: Lock) -> Result<(), eci_core::backend::LockingError> {
        let conn = self.0.get().map_err(LockingError::implementation)?;
        debug!("releasing lock {lock}");

        let locks_deleted = conn
            .execute(
                "delete from locks where lockid = :lockid",
                named_params! { ":lockid": lock.id()},
            )
            .map_err(LockingError::implementation)?;

        debug!("deleted locks on {locks_deleted} resources by releasing {lock}",);
        Ok(())
    }
}

pub(crate) fn create_lock_table(
    conn: &Pool<SqliteConnectionManager>,
) -> Result<(), rusqlite::Error> {
    conn.get().unwrap().execute_batch(
        "
        create table if not exists locks (
            lockid    text not null,
            entity    text not null,
            component text not null,
            locktype  text not null,
            expires   text not null
        ) strict;
    ",
    )
}

#[cfg(test)]
mod tests {
    use eci_core::{
        backend::{LockDescriptor, LockingBackend, LockingError, LockingMode},
        Entity,
    };

    use crate::SqliteBackend;
    const LOCK_TIME: std::time::Duration = std::time::Duration::from_secs(60);

    #[test]
    fn test_acquire_locking() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        conn.acquire_lock(
            entity,
            vec![
                LockDescriptor {
                    mode: LockingMode::Read,
                    name: "DebugComponentA".to_string(),
                },
                LockDescriptor {
                    mode: LockingMode::Read,
                    name: "DebugComponentA".to_string(),
                },
            ],
            LOCK_TIME,
        )
        .unwrap();
    }

    #[test]
    fn allow_multiple_read_locks() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock(
                entity,
                vec![
                    LockDescriptor {
                        mode: LockingMode::Read,
                        name: "DebugComponentA".to_string(),
                    },
                    LockDescriptor {
                        mode: LockingMode::Read,
                        name: "DebugComponentA".to_string(),
                    },
                ],
                LOCK_TIME,
            )
            .unwrap();

        let _b = conn
            .acquire_lock(
                entity,
                vec![
                    LockDescriptor {
                        mode: LockingMode::Read,
                        name: "DebugComponentA".to_string(),
                    },
                    LockDescriptor {
                        mode: LockingMode::Read,
                        name: "DebugComponentA".to_string(),
                    },
                ],
                LOCK_TIME,
            )
            .unwrap();
    }

    #[test]
    fn fail_on_multiple_write_locks() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock(
                entity,
                vec![LockDescriptor {
                    mode: LockingMode::Write,
                    name: "DebugComponentA".to_string(),
                }],
                LOCK_TIME,
            )
            .unwrap();

        assert_eq!(
            conn.acquire_lock(
                entity,
                vec![LockDescriptor {
                    mode: LockingMode::Write,
                    name: "DebugComponentA".to_string(),
                },],
                LOCK_TIME,
            )
            .unwrap_err()
            .to_string(),
            LockingError::Conflict(entity, "DebugComponentA".to_string(), LockingMode::Write)
                .to_string()
        );
    }

    #[test]
    fn allow_multiple_write_locks_on_different_components() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock(
                entity,
                vec![
                    LockDescriptor {
                        mode: LockingMode::Write,
                        name: "DebugComponentA".to_string(),
                    },
                    LockDescriptor {
                        mode: LockingMode::Write,
                        name: "DebugComponentB".to_string(),
                    },
                ],
                LOCK_TIME,
            )
            .unwrap();

        let _b = conn
            .acquire_lock(
                entity,
                vec![LockDescriptor {
                    mode: LockingMode::Write,
                    name: "DebugComponentC".to_string(),
                }],
                LOCK_TIME,
            )
            .unwrap();
    }

    #[test]
    fn fail_acquire_write_lock_while_read_locks_exist() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock(
                entity,
                vec![LockDescriptor {
                    mode: LockingMode::Read,
                    name: "DebugComponentA".to_string(),
                }],
                LOCK_TIME,
            )
            .unwrap();

        assert_eq!(
            conn.acquire_lock(
                entity,
                vec![LockDescriptor {
                    mode: LockingMode::Write,
                    name: "DebugComponentA".to_string(),
                },],
                LOCK_TIME,
            )
            .unwrap_err()
            .to_string(),
            LockingError::Conflict(entity, "DebugComponentA".to_string(), LockingMode::Write)
                .to_string()
        );
    }

    #[test]
    fn fail_acquire_read_lock_while_write_locks_exist() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock(
                entity,
                vec![LockDescriptor {
                    mode: LockingMode::Write,
                    name: "DebugComponentA".to_string(),
                }],
                LOCK_TIME,
            )
            .unwrap();

        assert_eq!(
            conn.acquire_lock(
                entity,
                vec![LockDescriptor {
                    mode: LockingMode::Read,
                    name: "DebugComponentA".to_string(),
                },],
                LOCK_TIME,
            )
            .unwrap_err()
            .to_string(),
            LockingError::Conflict(entity, "DebugComponentA".to_string(), LockingMode::Read)
                .to_string()
        );
    }
}
