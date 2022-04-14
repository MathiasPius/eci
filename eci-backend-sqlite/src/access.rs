use eci_core::backend::{
    AccessBackend, AccessError, ExtractionDescriptor, Format, SerializedComponent,
};
use rusqlite::{named_params};

use crate::SqliteBackend;

impl<F: Format> AccessBackend<F> for SqliteBackend {
    fn write_components(
        &self,
        entity: eci_core::Entity,
        components: Vec<SerializedComponent<F>>,
    ) -> Result<(), AccessError> {
        let mut conn = self.0.get().map_err(AccessError::implementation)?;
        let tx = conn.transaction().map_err(AccessError::implementation)?;

        for descriptor in components {
            let name = descriptor.name;
            let serialized_contents: Vec<u8> = descriptor.contents.into();

            let params = named_params! {
                ":entity": entity.to_string(),
                ":contents": serialized_contents,
            };

            // TODO: Should not be creating the table at this point in time but whatever.
            tx.execute_batch(&format!(
                "
            create table if not exists {name} (
                entity   text not null unique,
                contents blob not null
            );"
            ))
            .map_err(AccessError::implementation)?;

            if tx
                .execute(
                    &format!("insert into {name} (entity, contents) values(:entity, :contents)"),
                    params,
                )
                .map_err(AccessError::implementation)?
                != 1
            {
                return Err(AccessError::Conflict(entity, name.to_string()));
            };
        }

        tx.commit().map_err(AccessError::implementation)?;
        Ok(())
    }

    fn read_components(
        &self,
        entity: eci_core::Entity,
        descriptors: Vec<ExtractionDescriptor>,
    ) -> Result<Vec<Option<SerializedComponent<F>>>, AccessError> {
        let mut conn = self.0.get().map_err(AccessError::implementation)?;
        let tx = conn.transaction().map_err(AccessError::implementation)?;

        let mut components = Vec::new();
        for descriptor in descriptors {
            let name = descriptor.name;

            let params = named_params! {
                ":entity": entity.to_string(),
            };

            components.push(
                tx.query_row(
                    &format!(
                        "
                    select contents from {name} 
                    where entity = :entity
                "
                    ),
                    params,
                    |row| {
                        Ok(SerializedComponent::<F> {
                            contents: F::Data::from(row.get(0)?),
                            name,
                        })
                    },
                )
                .ok(),
            );
        }

        Ok(components)
    }
}

#[cfg(test)]
mod tests {
    use eci_core::{
        backend::{AccessBackend, ExtractionDescriptor, Format, SerializedComponent},
        Entity,
    };
    use eci_format_json::Json;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct DebugComponentA {
        content: String,
    }

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct DebugComponentB {
        content: String,
    }

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct DebugComponentC {
        content: String,
    }

    use crate::SqliteBackend;

    #[test]
    fn insert_disparate_components() {
        let conn = SqliteBackend::in_memory().unwrap();
        let entity = Entity::new();

        conn.write_components(
            entity,
            vec![
                SerializedComponent::<Json> {
                    contents: Json::serialize(DebugComponentA {
                        content: "Hello".to_string(),
                    })
                    .unwrap(),
                    name: "DebugComponentA".to_string(),
                },
                SerializedComponent::<Json> {
                    contents: Json::serialize(DebugComponentB {
                        content: "Hello".to_string(),
                    })
                    .unwrap(),
                    name: "DebugComponentB".to_string(),
                },
            ],
        )
        .unwrap();
    }

    #[test]
    fn fail_on_duplicate_components() {
        let conn = SqliteBackend::in_memory().unwrap();
        let entity = Entity::new();

        conn.write_components(
            entity,
            vec![
                SerializedComponent::<Json> {
                    contents: Json::serialize(DebugComponentA {
                        content: "Hello".to_string(),
                    })
                    .unwrap(),
                    name: "DebugComponentA".to_string(),
                },
                SerializedComponent::<Json> {
                    contents: Json::serialize(DebugComponentA {
                        content: "Hello".to_string(),
                    })
                    .unwrap(),
                    name: "DebugComponentA".to_string(),
                },
            ],
        )
        .unwrap_err();
    }

    #[test]
    fn read_components() {
        let conn = SqliteBackend::in_memory().unwrap();
        let entity = Entity::new();

        let a = DebugComponentA {
            content: "Hello".to_string(),
        };

        let b = DebugComponentB {
            content: "World".to_string(),
        };

        conn.write_components(
            entity,
            vec![
                SerializedComponent::<Json> {
                    contents: Json::serialize(&a).unwrap(),
                    name: "DebugComponentA".to_string(),
                },
                SerializedComponent::<Json> {
                    contents: Json::serialize(&b).unwrap(),
                    name: "DebugComponentB".to_string(),
                },
            ],
        )
        .unwrap();

        let comps: Vec<Option<SerializedComponent<Json>>> = conn
            .read_components(
                entity,
                vec![
                    ExtractionDescriptor {
                        name: "DebugComponentA".to_string(),
                    },
                    ExtractionDescriptor {
                        name: "DebugComponentB".to_string(),
                    },
                ],
            )
            .unwrap();

        let ax = Json::deserialize(&comps[0].as_ref().unwrap().contents).unwrap();
        let bx = Json::deserialize(&comps[1].as_ref().unwrap().contents).unwrap();

        assert_eq!(a, ax);
        assert_eq!(b, bx);
    }

    #[test]
    fn read_same_component_twice() {
        // While not exactly useful, there's no real reason why you shouldn't be allowed to
        // read the same component twice within a single query
        let conn = SqliteBackend::in_memory().unwrap();
        let entity = Entity::new();

        let a = DebugComponentA {
            content: "Hello".to_string(),
        };

        conn.write_components(
            entity,
            vec![SerializedComponent::<Json> {
                contents: Json::serialize(&a).unwrap(),
                name: "DebugComponentA".to_string(),
            }],
        )
        .unwrap();

        let comps: Vec<Option<SerializedComponent<Json>>> = conn
            .read_components(
                entity,
                vec![
                    ExtractionDescriptor {
                        name: "DebugComponentA".to_string(),
                    },
                    ExtractionDescriptor {
                        name: "DebugComponentA".to_string(),
                    },
                ],
            )
            .unwrap();

        let ax = Json::deserialize(&comps[0].as_ref().unwrap().contents).unwrap();
        let bx = Json::deserialize(&comps[1].as_ref().unwrap().contents).unwrap();

        assert_eq!(&a, &ax);
        assert_eq!(&a, &bx);
    }
}
