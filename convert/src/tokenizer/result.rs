use crate::util::SourcePosition;

pub type TokenizeResult<T> = Result<T, TokenizeError>;

#[derive(Debug)]
pub struct TokenizeError {
    pub message: String,
    pub source_position: SourcePosition,
}
