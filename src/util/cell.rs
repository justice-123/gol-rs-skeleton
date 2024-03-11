#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GolCell {
    pub x: usize,
    pub y: usize,
}

impl std::fmt::Display for GolCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
