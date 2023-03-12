use crate::tokenizer::state::block_separator::BlockSeparatorState;
use crate::tokenizer::state::emphasis::EmphasisState;
use crate::tokenizer::state::heading_level::HeadingLevelState;
use crate::tokenizer::state::result::{StateProcessResult, StateStackOperation};
use crate::tokenizer::state::text::TextState;
use crate::tokenizer::state::State;
use crate::tokenizer::Tokenizer;

pub(crate) struct StartState;

impl StartState {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl State for StartState {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult {
        match c {
            '\r' => return StateProcessResult::new(),
            _ => {
                let next_state: Box<dyn State> = match c {
                    '\n' => Box::new(BlockSeparatorState::new()),
                    '#' => Box::new(HeadingLevelState::new()),
                    '*' => Box::new(EmphasisState::new()),
                    _ => {
                        tokenizer.mark_char_as_unconsumed(c);
                        Box::new(TextState::new())
                    }
                };

                StateProcessResult::new()
                    .with_state_stack_operation(StateStackOperation::Push(next_state))
            }
        }
    }

    fn name(&self) -> &str {
        "StartState"
    }
}
