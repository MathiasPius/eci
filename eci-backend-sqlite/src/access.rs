use eci_core::{
    backend::{
        AccessBackend, AccessError, ExtractionDescriptor, Format, FromSerializedComponent,
        SerializedComponent, ToSerializedComponent,
    },
    Version,
};
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

    fn read_components<F, T>(&self, entity: eci_core::Entity) -> Result<T, AccessError>
    where
        F: Format,
        T: FromSerializedComponent<F>,
    {
        let mut conn = self.0.get().map_err(AccessError::implementation)?;
        let tx = conn.transaction().map_err(AccessError::implementation)?;

        let mut components = Vec::new();
        for descriptor in T::to_component_descriptor() {
            components.push(match descriptor {
                // If the descriptor is for an entity, we just put an empty
                // serialized component in there. When T::from_serialized_component
                // is run, it just ignores the SerializedComponent structure entirely
                // and passes on the input entity.
                ExtractionDescriptor::Entity => SerializedComponent::<F> {
                    contents: F::Data::from(vec![]),
                    name: "",
                    version: Version::new(1, 0, 0),
                },
                ExtractionDescriptor::Component(descriptor) => {
                    let name = descriptor.name;

                    let params = named_params! {
                        ":entity": entity.to_string(),
                        ":version": descriptor.version.to_string(),
                    };

                    tx.query_row(
                        &format!(
                            "
                    select contents from {name} 
                    where entity = :entity
                    and version = :version
                "
                        ),
                        params,
                        |row| {
                            Ok(SerializedComponent::<F> {
                                contents: F::Data::from(row.get(0)?),
                                name,
                                version: descriptor.version,
                            })
                        },
                    )
                    .map_err(AccessError::implementation)?
                }
            });
        }

        T::from_serialized_components(entity, components.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use eci_core::{backend::AccessBackend, Component, Entity};
    use eci_format_json::Json;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Component, Serialize, Deserialize, PartialEq, Eq)]
    struct DebugComponentA {
        content: String,
    }

    #[derive(Clone, Debug, Component, Serialize, Deserialize, PartialEq, Eq)]
    struct DebugComponentB {
        content: String,
    }

    #[derive(Clone, Debug, Component, Serialize, Deserialize, PartialEq, Eq)]
    struct DebugComponentC {
        content: String,
    }

    use crate::SqliteBackend;

    #[test]
    fn insert_disparate_components() {
        let conn = SqliteBackend::in_memory().unwrap();
        let entity = Entity::new();

        conn.write_components::<Json, (DebugComponentA, DebugComponentB)>(
            entity,
            (
                DebugComponentA {
                    content: "Hello".to_string(),
                },
                DebugComponentB {
                    content: "World".to_string(),
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
                    content: "Hello".to_string(),
                },
                DebugComponentA {
                    content: "World".to_string(),
                },
            ),
        )
        .unwrap_err();
    }

    #[test]
    fn read_components() {
        let conn = SqliteBackend::in_memory().unwrap();
        let entity = Entity::new();

        let input_components = (
            DebugComponentA {
                content: "Hello".to_string(),
            },
            DebugComponentB {
                content: "World".to_string(),
            },
        );

        conn.write_components::<Json, (DebugComponentA, DebugComponentB)>(
            entity,
            input_components.clone(),
        )
        .unwrap();

        let comps: (DebugComponentA, DebugComponentB) = conn
            .read_components::<Json, (DebugComponentA, DebugComponentB)>(entity)
            .unwrap();

        assert_eq!(comps, input_components);

        println!("{:#?}", comps);
    }

    #[test]
    fn read_same_component_twice() {
        // While not exactly useful, there's no real reason why you shouldn't be allowed to
        // read the same component twice within a single query
        let conn = SqliteBackend::in_memory().unwrap();
        let entity = Entity::new();

        let input_components = DebugComponentA {
            content: "Hello".to_string(),
        };

        conn.write_components::<Json, DebugComponentA>(entity, input_components.clone())
            .unwrap();

        let comps: (DebugComponentA, DebugComponentA) = conn
            .read_components::<Json, (DebugComponentA, DebugComponentA)>(entity)
            .unwrap();

        assert_eq!(comps.0, input_components);
        assert_eq!(comps.1, input_components);

        println!("{:#?}", comps);
    }

    #[test]
    fn read_entity_as_component() {
        let conn = SqliteBackend::in_memory().unwrap();
        let entity = Entity::new();

        let component = DebugComponentA {
            content: "Hello".to_string(),
        };

        conn.write_components::<Json, DebugComponentA>(entity, component.clone())
            .unwrap();

        let comps: (Entity, DebugComponentA) = conn
            .read_components::<Json, (Entity, DebugComponentA)>(entity)
            .unwrap();

        assert_eq!(entity, comps.0);
        assert_eq!(component, comps.1);
    }
}
