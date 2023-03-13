use TokenKind::Text;

use crate::source_position::SourcePosition;
use crate::source_span::SourceSpan;
use crate::tokenizer::state::result::StateProcessResult;
use crate::tokenizer::state::State;
use crate::tokenizer::token::{Token, TokenKind};
use crate::tokenizer::Tokenizer;

pub(crate) struct TextState {
    start_position: SourcePosition,
    text: String,
}

impl TextState {
    pub(crate) fn new(source_position: SourcePosition) -> Self {
        Self {
            start_position: SourcePosition {
                line: source_position.line,
                column: source_position.column - 1,
            },
            text: String::new(),
        }
    }

    fn create_token(&mut self) -> Token {
        let source_span = SourceSpan {
            start: self.start_position.clone(),
            end: SourcePosition {
                line: self.start_position.line,
                column: self.start_position.column + self.text.len(),
            },
        };

        Token::new(Text(self.text.clone()), source_span)
    }
}

impl State for TextState {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult {
        match c {
            '\n' | '*' => {
                tokenizer.mark_char_as_unconsumed(c);

                let token = self.create_token();
                return StateProcessResult::new().with_token(token);
            }
            // TODO Add function start
            // TODO Add link
            _ => self.text.push(c),
        }

        StateProcessResult::new()
    }

    fn end_of_input(&mut self, _tokenizer: &mut Tokenizer) -> StateProcessResult {
        let token = self.create_token();
        return StateProcessResult::new().with_token(token);
    }

    fn name(&self) -> &str {
        "TextState"
    }
}
