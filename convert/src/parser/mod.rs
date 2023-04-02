//! Parse various Markdown blocks.

mod block;
mod result;

pub(crate) use block::ParsedBlock;
pub(crate) use result::{ParseError, ParseResult};

use crate::categorizer::CategorizedBlock;

pub(crate) struct BlockParser;

impl BlockParser {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn parse(&self, categorized_block: CategorizedBlock) -> ParseResult<ParsedBlock> {
        todo!()
    }
}
