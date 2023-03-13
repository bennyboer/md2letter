use std::error::Error;

use crate::tokenizer::state::State;
use crate::tokenizer::token::Token;

pub(crate) struct StateProcessResult {
    pub(crate) token: Option<Token>,
    pub(crate) err: Option<Box<dyn Error>>,
    pub(crate) state_stack_operation: StateStackOperation,
}

impl StateProcessResult {
    pub(crate) fn new() -> Self {
        Self {
            token: None,
            err: None,
            state_stack_operation: StateStackOperation::None,
        }
    }

    pub(crate) fn with_token(self, token: Token) -> Self {
        Self {
            token: Some(token),
            err: self.err,
            state_stack_operation: self.state_stack_operation,
        }
    }

    pub(crate) fn _with_err(self, err: Box<dyn Error>) -> Self {
        Self {
            token: self.token,
            err: Some(err),
            state_stack_operation: self.state_stack_operation,
        }
    }

    pub(crate) fn with_state_stack_operation(
        self,
        state_stack_operation: StateStackOperation,
    ) -> Self {
        Self {
            token: self.token,
            err: self.err,
            state_stack_operation,
        }
    }
}

pub(crate) enum StateStackOperation {
    Push(Box<dyn State>),
    Pop,
    Replace(Box<dyn State>),
    None,
}
