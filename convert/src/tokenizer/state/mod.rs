pub(crate) use result::StateStackOperation;
pub(crate) use start::StartState;

use crate::tokenizer::state::result::StateProcessResult;
use crate::tokenizer::Tokenizer;

mod block_separator;
mod heading_level;
mod horizontal_rule;
mod result;
mod start;
mod text;

pub(crate) trait State {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult;
    fn end_of_input(&mut self, tokenizer: &mut Tokenizer) -> StateProcessResult;
    fn name(&self) -> &str;
}
