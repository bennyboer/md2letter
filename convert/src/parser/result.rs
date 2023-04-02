use crate::source_position::SourcePosition;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub source_position: SourcePosition,
}
