use TokenKind::{BoldEmphasis, ItalicEmphasis};

use crate::source_position::SourcePosition;
use crate::source_span::SourceSpan;
use crate::tokenizer::state::result::StateProcessResult;
use crate::tokenizer::state::State;
use crate::tokenizer::token::{Token, TokenKind};
use crate::tokenizer::Tokenizer;

pub(crate) struct EmphasisState {
    start_position: SourcePosition,
}

impl EmphasisState {
    pub(crate) fn new(source_position: SourcePosition) -> Self {
        Self {
            start_position: SourcePosition {
                line: source_position.line,
                column: source_position.column - 1,
            },
        }
    }
}

impl State for EmphasisState {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult {
        let emphasis = match c {
            '*' => BoldEmphasis,
            _ => {
                tokenizer.mark_char_as_unconsumed(c);

                ItalicEmphasis
            }
        };

        let source_span = SourceSpan {
            start: self.start_position.clone(),
            end: tokenizer.source_position(),
        };
        let token = Token::new(emphasis, source_span);
        StateProcessResult::new().with_token(token)
    }

    fn end_of_input(&mut self, _tokenizer: &mut Tokenizer) -> StateProcessResult {
        StateProcessResult::new()
    }

    fn name(&self) -> &str {
        "EmphasisState"
    }
}
