use TokenKind::BlockSeparator;

use crate::source_position::SourcePosition;
use crate::source_span::SourceSpan;
use crate::tokenizer::state::result::{StateProcessResult, StateStackOperation};
use crate::tokenizer::state::State;
use crate::tokenizer::token::{Token, TokenKind};
use crate::tokenizer::Tokenizer;

pub(crate) struct BlockSeparatorState {
    start_position: SourcePosition,
    newline_count: usize,
}

impl BlockSeparatorState {
    pub(crate) fn new(source_position: SourcePosition) -> Self {
        Self {
            start_position: source_position,
            newline_count: 1,
        }
    }
}

impl State for BlockSeparatorState {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult {
        match c {
            '\n' => self.newline_count += 1,
            ' ' | '\t' => {}
            _ => {
                tokenizer.mark_char_as_unconsumed(c);

                let mut result =
                    StateProcessResult::new().with_state_stack_operation(StateStackOperation::Pop);

                if self.newline_count >= 2 {
                    result = result.with_token(Token::new(
                        BlockSeparator,
                        SourceSpan {
                            start: self.start_position.clone(),
                            end: SourcePosition {
                                line: self.start_position.line + self.newline_count - 1,
                                column: 1,
                            },
                        },
                    ));
                }

                return result;
            }
        };

        StateProcessResult::new()
    }

    fn end_of_input(&mut self, _tokenizer: &mut Tokenizer) -> StateProcessResult {
        StateProcessResult::new()
    }

    fn name(&self) -> &str {
        "BlockSeparatorState"
    }
}
