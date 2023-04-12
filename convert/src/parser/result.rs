use crate::categorizer::BlockKind;
use crate::util::SourcePosition;

pub(crate) type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub(crate) struct ParseError {
    pub message: String,
    pub source_position: SourcePosition,
}
