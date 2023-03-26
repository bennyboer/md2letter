use crate::source_position::SourcePosition;

pub type TokenizeResult<T> = Result<T, TokenizeError>;

#[derive(Debug)]
pub struct TokenizeError {
    pub message: String,
    pub source_position: SourcePosition,
}
