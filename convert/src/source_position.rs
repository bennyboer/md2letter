#[derive(Debug, Clone, PartialEq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

impl SourcePosition {
    pub fn zero() -> Self {
        Self { line: 1, column: 1 }
    }

    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}
