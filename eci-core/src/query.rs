use std::marker::PhantomData;

use crate::{backend::StorageBackend, Component, Entity};

pub trait Fetch: Sized {
    fn get(world: &impl StorageBackend, entity: Entity) -> Option<Self>;
}

impl<T: Component> Fetch for T {
    fn get(world: &impl StorageBackend, entity: Entity) -> Option<Self> {
        world.get(entity).map(|c| c.component)
    }
}

impl Fetch for Entity {
    fn get(_: &impl StorageBackend, entity: Entity) -> Option<Self> {
        Some(entity)
    }
}

pub(crate) trait Fetchable<Select: Fetch> {
    fn get_all(&self) -> Vec<Select>;
}

pub trait Queryable {
    fn query<Select: Fetch, Where>(&self) -> Query<Select, Where>;
}

impl<B, Select> Fetchable<Select> for B
where
    B: StorageBackend,
    Select: Fetch,
{
    fn get_all(&self) -> Vec<Select> {
        self.entities()
            .iter()
            .filter_map(|entity| Select::get(self, *entity))
            .collect()
    }
}

pub struct Query<'world, Select: Fetch, Where = ()> {
    source: &'world dyn Fetchable<Select>,
    _select: PhantomData<Select>,
    _where: PhantomData<Where>,
}

impl<'world, Select, Where> Query<'world, Select, Where>
where
    Select: Fetch,
{
    pub(crate) fn in_world(source: &'world dyn Fetchable<Select>) -> Self {
        Query {
            source: source,
            _select: PhantomData::default(),
            _where: PhantomData::default(),
        }
    }

    pub fn iter(&self) -> Vec<Select> {
        self.source.get_all()
    }
}
