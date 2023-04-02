use std::collections::HashMap;

use crate::source_span::SourceSpan;

pub(crate) type FunctionName = String;

pub(crate) type ParameterName = String;
pub(crate) type ParameterValue = String;
pub(crate) type FunctionParameters = HashMap<ParameterName, ParameterValue>;

pub(crate) struct FunctionBlock {
    name: FunctionName,
    parameters: FunctionParameters,
    span: SourceSpan,
}
