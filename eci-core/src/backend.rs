use std::{error::Error, fmt::Display};

use semver::Version;

use crate::{Component, Entity};

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
    Conflict(&'static str, Version, LockingMode),
}

impl Display for LockingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LockingError::Implementation(inner) => {
                write!(f, "error while acquiring lock: {}", inner)
            }
            LockingError::Conflict(component, version, mode) => write!(
                f,
                "conflicting lock for {component} ({version}) while acquiring {mode} lock"
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

pub trait LockingBackend {
    type Lock;

    fn acquire_lock<T: ToLockDescriptor>(&self, entity: Entity)
        -> Result<Self::Lock, LockingError>;
    fn release_lock(&self, lock: Self::Lock) -> Result<(), LockingError>;
}

pub struct LockDescriptor {
    pub mode: LockingMode,
    pub name: &'static str,
    pub version: Version,
}

// TODO: It ought to be possible to make this known at compile time...
pub trait ToLockDescriptor {
    fn to_lock_descriptor() -> Vec<LockDescriptor>;
}

impl<T: Component> ToLockDescriptor for &T {
    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        vec![LockDescriptor {
            mode: LockingMode::Read,
            name: T::NAME,
            version: T::VERSION,
        }]
    }
}

impl<T: Component> ToLockDescriptor for &mut T {
    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        vec![LockDescriptor {
            mode: LockingMode::Write,
            name: T::NAME,
            version: T::VERSION,
        }]
    }
}

impl<A: ToLockDescriptor, B: ToLockDescriptor> ToLockDescriptor for (A, B) {
    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        let mut first = A::to_lock_descriptor();
        first.extend(B::to_lock_descriptor());
        first
    }
}

impl<A: ToLockDescriptor, B: ToLockDescriptor, C: ToLockDescriptor> ToLockDescriptor for (A, B, C) {
    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        let mut first = <(A, B)>::to_lock_descriptor();
        first.extend(C::to_lock_descriptor());
        first
    }
}

impl<A: ToLockDescriptor, B: ToLockDescriptor, C: ToLockDescriptor, D: ToLockDescriptor>
    ToLockDescriptor for (A, B, C, D)
{
    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        let mut first = <(A, B, C)>::to_lock_descriptor();
        first.extend(D::to_lock_descriptor());
        first
    }
}

impl<
        A: ToLockDescriptor,
        B: ToLockDescriptor,
        C: ToLockDescriptor,
        D: ToLockDescriptor,
        E: ToLockDescriptor,
    > ToLockDescriptor for (A, B, C, D, E)
{
    fn to_lock_descriptor() -> Vec<LockDescriptor> {
        let mut first = <(A, B, C, E)>::to_lock_descriptor();
        first.extend(E::to_lock_descriptor());
        first
    }
}
