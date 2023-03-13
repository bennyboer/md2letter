use crate::source_position::SourcePosition;

#[derive(Debug, PartialEq)]
pub struct SourceSpan {
    /// Start of the source span (inclusive).
    pub start: SourcePosition,

    /// End of source span (exclusive).
    pub end: SourcePosition,
}
