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
            _ => {
                let source_position = tokenizer.source_position();

                let next_state: Box<dyn State> = match c {
                    '\n' => Box::new(BlockSeparatorState::new(source_position)),
                    '#' => Box::new(HeadingLevelState::new(source_position)),
                    '*' => Box::new(EmphasisState::new(source_position)),
                    _ => {
                        tokenizer.mark_char_as_unconsumed(c);
                        Box::new(TextState::new(source_position))
                    }
                };

                StateProcessResult::new()
                    .with_state_stack_operation(StateStackOperation::Push(next_state))
            }
        }
    }

    fn end_of_input(&mut self, _tokenizer: &mut Tokenizer) -> StateProcessResult {
        StateProcessResult::new()
    }

    fn name(&self) -> &str {
        "StartState"
    }
}
