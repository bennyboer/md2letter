use std::collections::HashMap;

pub(crate) type FunctionName = String;

pub(crate) type ParameterName = String;
pub(crate) type ParameterValue = String;
pub(crate) type FunctionParameters = HashMap<ParameterName, ParameterValue>;

#[derive(Debug)]
pub(crate) struct FunctionBlock {
    name: FunctionName,
    parameters: FunctionParameters,
}

impl FunctionBlock {
    pub fn new(name: FunctionName, parameters: FunctionParameters) -> Self {
        Self { name, parameters }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parameters(&self) -> &FunctionParameters {
        &self.parameters
    }
}
