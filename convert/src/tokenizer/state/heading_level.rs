use crate::tokenizer::state::result::{StateProcessResult, StateStackOperation};
use crate::tokenizer::state::State;
use crate::tokenizer::token::Token;
use crate::tokenizer::Tokenizer;

pub(crate) struct HeadingLevelState {
    level: usize,
}

impl HeadingLevelState {
    pub(crate) fn new() -> Self {
        Self { level: 0 }
    }
}

impl State for HeadingLevelState {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult {
        match c {
            '#' => self.level += 1,
            _ => {
                tokenizer.mark_char_as_unconsumed(c);

                return StateProcessResult::new()
                    .with_state_stack_operation(StateStackOperation::Pop)
                    .with_token(Token::HeadingLevel(self.level));
            }
        };

        StateProcessResult::new()
    }

    fn name(&self) -> &str {
        "HeadingLevelState"
    }
}
