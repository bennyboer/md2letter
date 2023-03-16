use crate::source_position::SourcePosition;
use crate::source_span::SourceSpan;
use crate::tokenizer::state::result::{StateProcessResult, StateStackOperation};
use crate::tokenizer::state::State;
use crate::tokenizer::token::Token;
use crate::tokenizer::token::TokenKind::HorizontalRule;
use crate::tokenizer::Tokenizer;

pub(crate) struct HorizontalRuleState {
    start_position: SourcePosition,
    start_char: char,
    char_count: usize,
}

impl HorizontalRuleState {
    pub(crate) fn new(source_position: SourcePosition, start_char: char) -> Self {
        Self {
            start_position: SourcePosition {
                line: source_position.line,
                column: source_position.column - 1,
            },
            start_char,
            char_count: 1,
        }
    }

    fn create_token(&mut self) -> Option<Token> {
        if self.char_count >= 2 {
            let source_span = SourceSpan {
                start: self.start_position.clone(),
                end: SourcePosition {
                    line: self.start_position.line,
                    column: self.start_position.column + self.char_count,
                },
            };
            let token = Token::new(HorizontalRule, source_span);

            return Some(token);
        }

        None
    }
}

impl State for HorizontalRuleState {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult {
        let char_to_match = self.start_char;

        if c == char_to_match {
            self.char_count += 1;
            return StateProcessResult::new();
        }

        match c {
            '\n' => {
                tokenizer.mark_char_as_unconsumed(c);

                return self
                    .create_token()
                    .map(|token| {
                        StateProcessResult::new()
                            .with_state_stack_operation(StateStackOperation::Pop)
                            .with_token(token)
                    })
                    .unwrap_or_else(|| {
                        StateProcessResult::new()
                            .with_state_stack_operation(StateStackOperation::Pop)
                    });
            }
            ' ' | '\t' => {}
            _ => {
                tokenizer.mark_char_as_unconsumed(c);
                return StateProcessResult::new()
                    .with_state_stack_operation(StateStackOperation::Pop);
            }
        }

        StateProcessResult::new()
    }

    fn end_of_input(&mut self, _tokenizer: &mut Tokenizer) -> StateProcessResult {
        let token = self.create_token();

        StateProcessResult::new()
    }

    fn name(&self) -> &str {
        "HorizontalRuleState"
    }
}
