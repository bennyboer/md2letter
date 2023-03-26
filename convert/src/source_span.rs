use crate::source_position::SourcePosition;

#[derive(Debug, PartialEq, Clone)]
pub struct SourceSpan {
    /// Start of the source span (inclusive).
    pub start: SourcePosition,

    /// End of source span (exclusive).
    pub end: SourcePosition,
}

impl SourceSpan {
    pub fn new(start: SourcePosition, end: SourcePosition) -> Self {
        Self { start, end }
    }
}
