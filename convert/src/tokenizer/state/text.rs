use crate::tokenizer::state::result::StateProcessResult;
use crate::tokenizer::state::State;
use crate::tokenizer::token::Token;
use crate::tokenizer::Tokenizer;

pub(crate) struct TextState {
    text: String,
}

impl TextState {
    pub(crate) fn new() -> Self {
        Self {
            text: String::new(),
        }
    }
}

impl State for TextState {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult {
        match c {
            '\n' | '*' => {
                tokenizer.mark_char_as_unconsumed(c);

                return StateProcessResult::new().with_token(Token::Text(self.text.clone()));
            }
            '\r' => {}
            // TODO Add function start
            // TODO Add link
            _ => self.text.push(c),
        }

        StateProcessResult::new()
    }

    fn name(&self) -> &str {
        "TextState"
    }
}
