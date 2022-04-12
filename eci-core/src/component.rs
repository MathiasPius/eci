use crate::Version;
use serde::{Deserialize, Serialize};

pub trait Component {
    const NAME: &'static str;
    const VERSION: Version;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn version(&self) -> Version {
        Self::VERSION
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugComponentA {
    pub content: Option<String>,
}

impl Component for DebugComponentA {
    const NAME: &'static str = "DebugComponentA";
    const VERSION: Version = Version::new(1, 0, 0);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugComponentB {
    pub content: Option<String>,
}

impl Component for DebugComponentB {
    const NAME: &'static str = "DebugComponentB";
    const VERSION: Version = Version::new(1, 0, 0);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebugComponentC {
    pub content: Option<String>,
}

impl Component for DebugComponentC {
    const NAME: &'static str = "DebugComponentC";
    const VERSION: Version = Version::new(1, 0, 0);
}
