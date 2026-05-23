use crate::compiler::RatSource;

// #[derive(Eq)]
pub struct Render {
    source: RatSource,
}

impl Render {
    pub fn init(source: RatSource) -> Self {
        Render { source }
    }
}
