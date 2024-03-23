use std::fmt::Display;
use num_traits::PrimInt;

/// CellCoord (Cell coordinate) represents the coordinate of a cell in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellCoord<T = usize>
    where T: PrimInt
{
    pub x: T,
    pub y: T,
}

impl<T: PrimInt> CellCoord<T> {
    /// Create a new cell coordinate.
    pub fn new(x: T, y: T) -> Self {
        CellCoord { x, y }
    }
}

impl<T> Display for CellCoord<T>
    where T: PrimInt + std::fmt::Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
