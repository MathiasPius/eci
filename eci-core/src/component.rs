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
