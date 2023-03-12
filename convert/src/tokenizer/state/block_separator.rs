use crate::tokenizer::state::result::{StateProcessResult, StateStackOperation};
use crate::tokenizer::state::State;
use crate::tokenizer::token::Token;
use crate::tokenizer::Tokenizer;

pub(crate) struct BlockSeparatorState {
    newline_count: usize,
}

impl BlockSeparatorState {
    pub(crate) fn new() -> Self {
        Self { newline_count: 1 }
    }
}

impl State for BlockSeparatorState {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult {
        match c {
            '\n' => self.newline_count += 1,
            ' ' | '\t' | '\r' => {}
            _ => {
                tokenizer.mark_char_as_unconsumed(c);

                let mut result =
                    StateProcessResult::new().with_state_stack_operation(StateStackOperation::Pop);

                println!(
                    "BlockSeparatorState: newline_count = {}",
                    self.newline_count
                );
                if self.newline_count >= 2 {
                    result = result.with_token(Token::BlockSeparator);
                }

                return result;
            }
        };

        StateProcessResult::new()
    }

    fn name(&self) -> &str {
        "BlockSeparatorState"
    }
}
