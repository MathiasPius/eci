use chrono::{Duration, Utc};
use eci_core::backend::{LockingBackend, LockingError, LockingMode, ToLockDescriptor};
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
        expires_in: std::time::Duration,
    ) -> Result<Self::Lock, LockingError> {
        let lockid = Uuid::new_v4();

        let mut conn = self.0.get().map_err(LockingError::implementation)?;

        debug!("starting lock transaction for lock {lockid}");
        let tx = conn.transaction().map_err(LockingError::implementation)?;

        for descriptor in T::to_lock_descriptor() {
            let params = named_params! {
                ":lockid": lockid.to_string(),
                ":entity": entity.to_string(),
                ":component": descriptor.name,
                ":version": descriptor.version.to_string(),
                ":expires": Utc::now() + Duration::from_std(expires_in).map_err(LockingError::implementation)?,
            };

            debug!(
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
                    entity,
                    descriptor.name,
                    descriptor.version,
                    descriptor.mode,
                ));
            };
        }

        tx.commit().map_err(LockingError::implementation)?;
        debug!("lock {lockid} transaction committed");

        Ok(SqliteLock(lockid))
    }

    fn release_lock(&self, lock: Self::Lock) -> Result<(), eci_core::backend::LockingError> {
        let conn = self.0.get().map_err(LockingError::implementation)?;
        debug!("releasing lock {lockid}", lockid = lock.0);

        let locks_deleted = conn
            .execute(
                "delete from locks where lockid = :lockid",
                named_params! { ":lockid": lock.0.to_string()},
            )
            .map_err(LockingError::implementation)?;

        debug!(
            "deleted locks on {locks_deleted} resources by releasing {lockid}",
            lockid = lock.0
        );
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
        Component, Entity, Version,
    };

    #[derive(Component)]
    struct DebugComponentA;

    #[derive(Component)]
    struct DebugComponentB;

    #[derive(Component)]
    struct DebugComponentC;

    use crate::SqliteBackend;
    const LOCK_TIME: std::time::Duration = std::time::Duration::from_secs(60);

    #[test]
    fn test_acquire_locking() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        conn.acquire_lock::<(&DebugComponentA, &DebugComponentA)>(entity, LOCK_TIME)
            .unwrap();
    }

    #[test]
    fn allow_multiple_read_locks() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock::<(&DebugComponentA, &DebugComponentA)>(entity, LOCK_TIME)
            .unwrap();
        let _b = conn
            .acquire_lock::<(&DebugComponentA, &DebugComponentA)>(entity, LOCK_TIME)
            .unwrap();
        let _c = conn
            .acquire_lock::<(&DebugComponentA, &DebugComponentA)>(entity, LOCK_TIME)
            .unwrap();
    }

    #[test]
    fn fail_on_multiple_write_locks() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock::<&mut DebugComponentA>(entity, LOCK_TIME)
            .unwrap();

        assert_eq!(
            conn.acquire_lock::<&mut DebugComponentA>(entity, LOCK_TIME)
                .unwrap_err()
                .to_string(),
            LockingError::Conflict(
                entity,
                "DebugComponentA",
                Version::new(1, 0, 0),
                LockingMode::Write
            )
            .to_string()
        );
    }

    #[test]
    fn allow_multiple_write_locks_on_different_components() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock::<(&mut DebugComponentA, &mut DebugComponentB)>(entity, LOCK_TIME)
            .unwrap();
        let _b = conn
            .acquire_lock::<&mut DebugComponentC>(entity, LOCK_TIME)
            .unwrap();
    }

    #[test]
    fn fail_acquire_write_lock_while_read_locks_exist() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock::<&DebugComponentA>(entity, LOCK_TIME)
            .unwrap();

        assert_eq!(
            conn.acquire_lock::<&mut DebugComponentA>(entity, LOCK_TIME)
                .unwrap_err()
                .to_string(),
            LockingError::Conflict(
                entity,
                "DebugComponentA",
                Version::new(1, 0, 0),
                LockingMode::Write
            )
            .to_string()
        );
    }

    #[test]
    fn fail_acquire_read_lock_while_write_locks_exist() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        let _a = conn
            .acquire_lock::<&mut DebugComponentA>(entity, LOCK_TIME)
            .unwrap();

        assert_eq!(
            conn.acquire_lock::<&DebugComponentA>(entity, LOCK_TIME)
                .unwrap_err()
                .to_string(),
            LockingError::Conflict(
                entity,
                "DebugComponentA",
                Version::new(1, 0, 0),
                LockingMode::Read
            )
            .to_string()
        );
    }
}
