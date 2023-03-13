use TokenKind::HeadingLevel;

use crate::source_position::SourcePosition;
use crate::source_span::SourceSpan;
use crate::tokenizer::state::result::{StateProcessResult, StateStackOperation};
use crate::tokenizer::state::State;
use crate::tokenizer::token::{Token, TokenKind};
use crate::tokenizer::Tokenizer;

pub(crate) struct HeadingLevelState {
    start_position: SourcePosition,
    level: usize,
}

impl HeadingLevelState {
    pub(crate) fn new(source_position: SourcePosition) -> Self {
        Self {
            start_position: SourcePosition {
                line: source_position.line,
                column: source_position.column - 1,
            },
            level: 0,
        }
    }
}

impl State for HeadingLevelState {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult {
        match c {
            '#' => self.level += 1,
            _ => {
                tokenizer.mark_char_as_unconsumed(c);

                let source_position = tokenizer.source_position();
                let source_span = SourceSpan {
                    start: self.start_position.clone(),
                    end: SourcePosition {
                        line: source_position.line,
                        column: source_position.column - 1,
                    },
                };
                let token = Token::new(HeadingLevel(self.level), source_span);

                return StateProcessResult::new()
                    .with_state_stack_operation(StateStackOperation::Pop)
                    .with_token(token);
            }
        };

        StateProcessResult::new()
    }

    fn end_of_input(&mut self, _tokenizer: &mut Tokenizer) -> StateProcessResult {
        StateProcessResult::new()
    }

    fn name(&self) -> &str {
        "HeadingLevelState"
    }
}
