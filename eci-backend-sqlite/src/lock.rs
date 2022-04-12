use chrono::{Duration, Utc};
use eci_core::backend::{LockingBackend, LockingError, LockingMode, ToLockDescriptor};
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
    :version   as version,
    'write'    as locktype,
    :expires   as expires
where not exists(
    select entity from locks
    where entity   = :entity
    and component  = :component
    and version    = :version
    and datetime(current_timestamp) < datetime(expires)
);";

const READ_LOCK: &'static str = "insert into locks
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
);";

impl LockingBackend for SqliteBackend {
    type Lock = SqliteLock;

    fn acquire_lock<T: ToLockDescriptor>(
        &self,
        entity: eci_core::Entity,
    ) -> Result<Self::Lock, LockingError> {
        let lockid = Uuid::new_v4();

        let mut conn = self.0.get().map_err(LockingError::implementation)?;
        let tx = conn.transaction().map_err(LockingError::implementation)?;

        for descriptor in T::to_lock_descriptor() {
            let params = named_params! {
                ":lockid": lockid.to_string(),
                ":entity": entity.to_string(),
                ":component": descriptor.name,
                ":version": descriptor.version.to_string(),
                ":expires": Utc::now() + Duration::hours(1),
            };

            println!(
                "acquiring {}-lock for {}({})",
                descriptor.mode, descriptor.name, descriptor.version
            );

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
                    descriptor.name,
                    descriptor.version,
                    descriptor.mode,
                ));
            };
        }

        tx.commit().map_err(LockingError::implementation)?;

        Ok(SqliteLock(lockid))
    }

    fn release_lock(&self, lock: Self::Lock) -> Result<(), eci_core::backend::LockingError> {
        let conn = self.0.get().map_err(LockingError::implementation)?;
        conn.execute(
            "delete from locks where lockid = :lockid",
            named_params! { ":lockid": lock.0.to_string()},
        )
        .map_err(LockingError::implementation)?;

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
            version   text not null,
            locktype  text not null,
            expires   text not null
        ) strict;
    ",
    )
}

#[cfg(test)]
mod tests {
    use eci_core::{
        backend::{LockingBackend, LockingError, LockingMode},
        component::{DebugComponentA, DebugComponentB, DebugComponentC},
        Entity, Version,
    };

    use crate::SqliteBackend;

    #[test]
    fn test_acquire_locking() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        conn.acquire_lock::<(&DebugComponentA, &DebugComponentA)>(entity)
            .unwrap();
    }

    #[test]
    fn allow_multiple_read_locks() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock::<(&DebugComponentA, &DebugComponentA)>(entity)
            .unwrap();
        let _b = conn
            .acquire_lock::<(&DebugComponentA, &DebugComponentA)>(entity)
            .unwrap();
        let _c = conn
            .acquire_lock::<(&DebugComponentA, &DebugComponentA)>(entity)
            .unwrap();
    }

    #[test]
    fn fail_on_multiple_write_locks() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn.acquire_lock::<&mut DebugComponentA>(entity).unwrap();

        assert_eq!(
            conn.acquire_lock::<&mut DebugComponentA>(entity)
                .unwrap_err()
                .to_string(),
            LockingError::Conflict("DebugComponentA", Version::new(1, 0, 0), LockingMode::Write)
                .to_string()
        );
    }

    #[test]
    fn allow_multiple_write_locks_on_different_components() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock::<(&mut DebugComponentA, &mut DebugComponentB)>(entity)
            .unwrap();
        let _b = conn.acquire_lock::<&mut DebugComponentC>(entity).unwrap();
    }

    #[test]
    fn fail_acquire_write_lock_while_read_locks_exist() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn.acquire_lock::<&DebugComponentA>(entity).unwrap();

        assert_eq!(
            conn.acquire_lock::<&mut DebugComponentA>(entity)
                .unwrap_err()
                .to_string(),
            LockingError::Conflict("DebugComponentA", Version::new(1, 0, 0), LockingMode::Write)
                .to_string()
        );
    }

    #[test]
    fn fail_acquire_read_lock_while_write_locks_exist() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn.acquire_lock::<&mut DebugComponentA>(entity).unwrap();

        assert_eq!(
            conn.acquire_lock::<&DebugComponentA>(entity)
                .unwrap_err()
                .to_string(),
            LockingError::Conflict("DebugComponentA", Version::new(1, 0, 0), LockingMode::Read)
                .to_string()
        );
    }
}
