use eci_core::backend::{Lock, LockingBackend, LockingError};
use std::fmt::Debug;

use crate::{refcast::RefCast, Extractor};

/// Automatically releases the contained lock upon Drop
pub(crate) struct DropLock {
    lock: Option<Lock>,
    backend: Box<dyn LockingBackend>,
}

impl DropLock {
    pub fn new(lock: Lock, backend: Box<dyn LockingBackend>) -> Self {
        DropLock {
            lock: Some(lock),
            backend,
        }
    }

    pub fn unlock(mut self) -> Result<(), LockingError> {
        if let Some(lock) = self.lock.take() {
            self.backend.release_lock(lock)
        } else {
            Ok(())
        }
    }
}

impl Drop for DropLock {
    fn drop(&mut self) {
        if let Some(lock) = self.lock.take() {
            println!("dropped lock: {lock}");
            self.backend.release_lock(lock).ok();
        }
    }
}

impl Debug for DropLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DropLock")
            .field("lock", &self.lock)
            .finish()
    }
}

/// Represents access to a locked resource.
pub struct Locked<T>
where
    T: Extractor,
{
    lock: DropLock,
    inner: <T as Extractor>::Owned,
}

impl<'a, T> Locked<T>
where
    T: Extractor,
    T: RefCast<'a, Owned = <T as Extractor>::Owned>,
{
    pub(crate) fn new(lock: DropLock, components: <T as Extractor>::Owned) -> Self {
        Locked {
            lock,
            inner: components,
        }
    }

    pub fn unlock(self) -> Result<(), LockingError> {
        self.lock.unlock()
    }

    pub fn deref(&'a mut self) -> T {
        <T as RefCast>::refcast(&mut self.inner)
    }
}

impl<T> Debug for Locked<T>
where
    T: Extractor,
    <T as Extractor>::Owned: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Locked")
            .field("lock", &self.lock)
            .field("inner", &self.inner)
            .finish()
    }
}
