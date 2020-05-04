#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Position {
    pub line: usize,
    pub col: usize,
}
