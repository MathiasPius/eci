use crate::Version;

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

#[derive(Debug)]
pub struct DebugString {
    pub content: String,
}

impl Component for DebugString {
    const NAME: &'static str = "DebugString";
    const VERSION: Version = Version::new(1, 0, 0);
}
