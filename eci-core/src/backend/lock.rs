use std::{error::Error, fmt::Display};

use uuid::Uuid;

use crate::Entity;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LockingMode {
    Read,
    Write,
}

impl Display for LockingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockingMode::Read => write!(f, "read"),
            LockingMode::Write => write!(f, "write"),
        }
    }
}

#[derive(Debug)]
pub enum LockingError {
    Implementation(Box<dyn Error>),
    Conflict(Entity, String, LockingMode),
}

impl Display for LockingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockingError::Implementation(inner) => {
                write!(f, "error while acquiring lock: {}", inner)
            }
            LockingError::Conflict(entity, component, mode) => write!(
                f,
                "conflicting lock for {entity}'s {component} while acquiring {mode} lock"
            ),
        }
    }
}

impl Error for LockingError {}

impl LockingError {
    pub fn implementation<T: Error + 'static>(err: T) -> Self {
        LockingError::Implementation(Box::new(err))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Lock(Uuid);

impl Lock {
    pub fn new() -> Lock {
        Lock(Uuid::new_v4())
    }

    pub fn id(&self) -> String {
        self.0.to_string()
    }
}

impl Display for Lock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait LockingBackend {
    fn acquire_lock(
        &self,
        entity: Entity,
        descriptors: Vec<LockDescriptor>,
        expires_in: std::time::Duration,
    ) -> Result<Lock, LockingError>;
    fn release_lock(&self, lock: Lock) -> Result<(), LockingError>;
}

pub struct LockDescriptor {
    pub mode: LockingMode,
    pub name: String,
}

/*
// TODO: It ought to be possible to make this known at compile time...
pub trait ToLockDescriptor {
    type Owned;
    fn to_lock_descriptor() -> Vec<LockDescriptor>;
}

impl<T: Component> ToLockDescriptor for &T {
    type Owned = T;
    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        vec![LockDescriptor {
            mode: LockingMode::Read,
            name: T::NAME.to_string(),
        }]
    }
}

impl<T: Component> ToLockDescriptor for &mut T {
    type Owned = T;
    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        vec![LockDescriptor {
            mode: LockingMode::Write,
            name: T::NAME,
        }]
    }
}

impl<A, B> ToLockDescriptor for (A, B)
where
    A: ToLockDescriptor,
    B: ToLockDescriptor,
{
    type Owned = (A::Owned, B::Owned);

    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        let mut first = A::to_lock_descriptor();
        first.extend(B::to_lock_descriptor());
        first
    }
}

impl<A, B, C> ToLockDescriptor for (A, B, C)
where
    A: ToLockDescriptor,
    B: ToLockDescriptor,
    C: ToLockDescriptor,
{
    type Owned = (A::Owned, B::Owned, C::Owned);

    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        let mut first = <(A, B)>::to_lock_descriptor();
        first.extend(C::to_lock_descriptor());
        first
    }
}

impl<A, B, C, D> ToLockDescriptor for (A, B, C, D)
where
    A: ToLockDescriptor,
    B: ToLockDescriptor,
    C: ToLockDescriptor,
    D: ToLockDescriptor,
{
    type Owned = (A::Owned, B::Owned, C::Owned, D::Owned);

    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        let mut first = <(A, B, C)>::to_lock_descriptor();
        first.extend(D::to_lock_descriptor());
        first
    }
}

impl<A, B, C, D, E> ToLockDescriptor for (A, B, C, D, E)
where
    A: ToLockDescriptor,
    B: ToLockDescriptor,
    C: ToLockDescriptor,
    D: ToLockDescriptor,
    E: ToLockDescriptor,
{
    type Owned = (A::Owned, B::Owned, C::Owned, D::Owned, E::Owned);

    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        let mut first = <(A, B, C, E)>::to_lock_descriptor();
        first.extend(E::to_lock_descriptor());
        first
    }
}
*/
