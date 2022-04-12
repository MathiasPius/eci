use eci_core::backend::{AccessBackend, AccessError, Format, ToSerializedComponent};
use rusqlite::named_params;

use crate::SqliteBackend;

impl AccessBackend for SqliteBackend {
    fn write_components<F, T>(
        &self,
        entity: eci_core::Entity,
        components: T,
    ) -> Result<(), AccessError>
    where
        F: Format,
        T: ToSerializedComponent<F>,
    {
        let mut conn = self.0.get().map_err(AccessError::implementation)?;
        let tx = conn.transaction().map_err(AccessError::implementation)?;

        for descriptor in components.to_serialized_components()? {
            let name = descriptor.name;
            let serialized_contents: Vec<u8> = descriptor.contents.into();

            let params = named_params! {
                ":entity": entity.to_string(),
                ":version": descriptor.version.to_string(),
                ":contents": serialized_contents,
            };

            if tx.execute(&format!(
                "INSERT INTO {name} (entity, version, component) VALUES(:entity, :version, :component)"
            ), params).map_err(AccessError::implementation)? != 1 {
                return Err(AccessError::Conflict(
                    entity,
                    name,
                    descriptor.version
                ))
            };
        }

        tx.commit().map_err(AccessError::implementation)?;
        Ok(())
    }
}
