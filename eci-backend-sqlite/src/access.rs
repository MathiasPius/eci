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

            // TODO: Should not be creating the table at this point in time but whatever.
            tx.execute_batch(&format!(
                "
            create table if not exists {name} (
                entity   text not null unique,
                version  text not null,
                contents blob not null
            );"
            ))
            .map_err(AccessError::implementation)?;

            if tx.execute(&format!(
                "insert into {name} (entity, version, contents) values(:entity, :version, :contents)"
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

#[cfg(test)]
mod tests {
    use eci_core::{
        backend::AccessBackend,
        component::{DebugComponentA, DebugComponentB},
        Entity,
    };
    use eci_format_json::Json;

    use crate::SqliteBackend;

    #[test]
    fn insert_disparate_components() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        conn.write_components::<Json, (DebugComponentA, DebugComponentB)>(
            entity,
            (
                DebugComponentA {
                    content: Some("Hello".to_string()),
                },
                DebugComponentB {
                    content: Some("World".to_string()),
                },
            ),
        )
        .unwrap();
    }

    #[test]
    fn fail_on_duplicate_components() {
        let conn = SqliteBackend::in_memory().unwrap();

        let entity = Entity::new();

        conn.write_components::<Json, (DebugComponentA, DebugComponentA)>(
            entity,
            (
                DebugComponentA {
                    content: Some("Hello".to_string()),
                },
                DebugComponentA {
                    content: Some("World".to_string()),
                },
            ),
        )
        .unwrap_err();
    }
}
