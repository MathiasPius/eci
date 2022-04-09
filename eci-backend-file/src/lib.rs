use eci_core::{
    backend::{ComponentStorage, Format, SerializeableBackend, StorageBackend},
    Component,
};

use std::{marker::PhantomData, path::PathBuf};

#[derive(Debug)]
pub struct FileBackend<B, F>
where
    B: StorageBackend + SerializeableBackend<F>,
    F: Format,
{
    file: PathBuf,
    backend: B,
    _format: PhantomData<F>,
}

impl<B, F> FileBackend<B, F>
where
    B: StorageBackend + SerializeableBackend<F>,
    F: Format,
    F::Type: From<Vec<u8>> + Into<Vec<u8>>,
{
    pub fn new<P: Into<PathBuf>>(backend: B, path: P, _format: F) -> Self {
        let wrapper = FileBackend {
            file: path.into(),
            backend,
            _format: PhantomData::default(),
        };

        FileBackend::save(&wrapper).unwrap();
        wrapper
    }

    fn save(&self) -> Result<(), std::io::Error> {
        let value = self.backend.save().unwrap();

        std::fs::write(&self.file, value.into())?;
        Ok(())
    }

    pub fn load<P: Into<PathBuf>>(path: P) -> Result<Self, std::io::Error> {
        let path = path.into();

        let source = std::fs::read(&path)?.into();
        let value = F::Type::from(source);

        Ok(FileBackend {
            file: path,
            backend: B::load(value).unwrap(),
            _format: PhantomData::default(),
        })
    }
}

impl<B, F> StorageBackend for FileBackend<B, F>
where
    B: StorageBackend + SerializeableBackend<F>,
    F: Format,
    F::Type: From<Vec<u8>> + Into<Vec<u8>>,
{
    fn spawn(&mut self) -> eci_core::Entity {
        self.backend.spawn()
    }

    fn update<T: Component>(&mut self, component: ComponentStorage<T>) -> T {
        let component = self.backend.update(component);
        FileBackend::save(&self).unwrap();
        component
    }

    fn insert<T: Component>(
        &mut self,
        entity: eci_core::Entity,
        component: T,
    ) -> ComponentStorage<T> {
        let component = self.backend.insert(entity, component);
        FileBackend::save(&self).unwrap();
        component
    }

    fn remove<T: Component>(&mut self, entity: eci_core::Entity) -> T {
        let component = self.backend.remove(entity);
        FileBackend::save(&self).unwrap();
        component
    }

    fn get<T: Component>(&self, entity: eci_core::Entity) -> Option<ComponentStorage<T>> {
        self.backend.get(entity)
    }

    fn entities(&self) -> Vec<eci_core::Entity> {
        self.backend.entities()
    }
}

#[cfg(test)]
mod tests {
    use eci_backend_json::{Json, JsonBackend};
    use eci_core::{component::DebugString, query::Queryable, World};

    use crate::FileBackend;

    #[test]
    fn test_save_and_load_of_json_backend() {
        let backing_file = tempfile::NamedTempFile::new().unwrap();

        // This tests boostrapping a new file-backed world.
        {
            // Configure a Json-formatted FileBackend
            let new_backend = FileBackend::new(JsonBackend::default(), backing_file.path(), Json);

            // Initialize the world with some entity
            let mut world = World::new(new_backend);

            world
                .spawn()
                .insert(DebugString {
                    content: "Hello world!".to_string(),
                })
                .id();
        }

        // This tests that the world was properly saved
        {
            let reuse_backend =
                FileBackend::<JsonBackend, Json>::load(backing_file.path()).unwrap();
            let world = World::new(reuse_backend);

            let debug_str = world.query::<DebugString, ()>().iter().pop().unwrap();
            assert_eq!(debug_str.content, "Hello world!");
            println!("{:?}", debug_str);
        }
    }
}
