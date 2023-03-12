use crate::tokenizer::state::result::StateProcessResult;
use crate::tokenizer::state::State;
use crate::tokenizer::token::Token;
use crate::tokenizer::Tokenizer;

pub(crate) struct EmphasisState;

impl EmphasisState {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl State for EmphasisState {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult {
        let emphasis = match c {
            '*' => Token::BoldEmphasis,
            _ => {
                tokenizer.mark_char_as_unconsumed(c);

                Token::ItalicEmphasis
            }
        };

        StateProcessResult::new().with_token(emphasis)
    }

    fn name(&self) -> &str {
        "EmphasisState"
    }
}
