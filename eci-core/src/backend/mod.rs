mod access;
mod lock;
pub use access::*;
pub use lock::*;

use crate::Entity;

pub trait JointBackend<F: Format>: AccessBackend<F> + LockingBackend {}

impl<T, F> JointBackend<F> for T
where
    F: Format,
    T: AccessBackend<F> + LockingBackend,
{
}

pub enum Backend<F: Format> {
    Split {
        locking: Box<dyn LockingBackend>,
        access: Box<dyn AccessBackend<F>>,
    },
    Joint {
        backend: Box<dyn JointBackend<F>>,
    },
}

impl<F: Format> AccessBackend<F> for Backend<F> {
    fn write_components(
        &self,
        entity: Entity,
        components: Vec<SerializedComponent<F>>,
    ) -> Result<(), AccessError> {
        match self {
            Backend::Split { locking: _, access } => access.write_components(entity, components),
            Backend::Joint { backend } => backend.write_components(entity, components),
        }
    }

    fn read_components(
        &self,
        entity: Entity,
        descriptors: Vec<ExtractionDescriptor>,
    ) -> Result<Vec<SerializedComponent<F>>, AccessError> {
        match self {
            Backend::Split { locking: _, access } => access.read_components(entity, descriptors),
            Backend::Joint { backend } => backend.read_components(entity, descriptors),
        }
    }
}
