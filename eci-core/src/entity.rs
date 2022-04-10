use std::fmt::{Display, Formatter};

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Entity(Uuid);

impl Entity {
    pub fn new() -> Entity {
        Entity(Uuid::new_v4())
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}