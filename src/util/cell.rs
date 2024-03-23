#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellCoord {
    pub x: usize,
    pub y: usize,
}

impl std::fmt::Display for CellCoord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
