use std::error::Error;

pub type TokenizeResult<T> = Result<T, Box<dyn Error>>;
