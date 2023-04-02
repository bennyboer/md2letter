//! Parse various Markdown blocks.

pub(crate) use block::ParsedBlock;
pub(crate) use result::{ParseError, ParseResult};

use crate::categorizer::{BlockKind, CategorizedBlock};
use crate::parser::text::TextParser;

mod block;
mod result;
mod text;

pub(crate) struct BlockParser;

impl BlockParser {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn parse(&self, categorized_block: CategorizedBlock) -> ParseResult<ParsedBlock> {
        match categorized_block.kind() {
            BlockKind::Text => TextParser::new(categorized_block).parse(),
            _ => Err(ParseError {
                block: categorized_block,
                message: "Parser for block not implemented yet".to_string(),
                source_position: categorized_block.span().start.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_fallback_to_text_parser_if_parsing_fails() {
        // TODO
    }

    // TODO More tests
}
