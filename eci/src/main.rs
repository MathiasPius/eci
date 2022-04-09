fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use eci_backend_json::JsonBackend;
    use eci_core::{component::DebugString, query::Queryable, World};

    #[test]
    fn test_query() {
        // Initialize the world with some entity
        let mut world = World::new(JsonBackend::default());

        world.spawn().insert(DebugString {
            content: "test 1".to_string(),
        });

        world.spawn().insert(DebugString {
            content: "test 2".to_string(),
        });

        for item in world.query::<DebugString, ()>().iter() {
            println!("{:#?}", item);
        }
    }
}
