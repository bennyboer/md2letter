use crate::categorizer::CategorizedBlock;
use crate::util::SourcePosition;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub struct ParseError {
    pub block: CategorizedBlock,
    pub message: String,
    pub source_position: SourcePosition,
}
