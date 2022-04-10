use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::{component::Component, Entity};

#[derive(Debug)]
pub struct ComponentStorage<T: Component> {
    pub entity: Entity,
    pub component: T,
}

impl<T> Deref for ComponentStorage<T>
where
    T: Component,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.component
    }
}

impl<T> DerefMut for ComponentStorage<T>
where
    T: Component,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}