use crate::parser::ParsedBlock;

pub(crate) type TransformResult<T> = Result<T, TransformError>;

#[derive(Debug)]
pub(crate) struct TransformError {
    pub message: String,
    pub block: ParsedBlock,
}
