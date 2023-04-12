//! Parse various Markdown blocks.

pub(crate) use block::ParsedBlock;
pub(crate) use result::{ParseError, ParseResult};

use crate::categorizer::{BlockKind, CategorizedBlock};
use crate::parser::heading::HeadingParser;
use crate::parser::list::ListParser;
use crate::parser::text::TextParser;

mod block;
mod heading;
mod list;
mod result;
mod text;

pub(crate) struct BlockParser;

impl BlockParser {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn parse(&self, categorized_block: CategorizedBlock) -> ParseResult<ParsedBlock> {
        let (kind, src, span) = categorized_block.consume();

        match kind {
            BlockKind::Text => TextParser::new(src, span.clone()).parse(),
            BlockKind::Heading => HeadingParser::new(src, span.clone()).parse(),
            BlockKind::List => ListParser::new(src, span.clone()).parse(),
            _ => Err(ParseError {
                message: "Parser for block not implemented yet".to_string(),
                source_position: span.start.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::{SourcePosition, SourceSpan};

    use super::*;

    #[test]
    fn should_parse_text_block() {
        let src = "This is a paragraph.";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 21));
        let categorized_block = CategorizedBlock::new(BlockKind::Text, src.to_string(), span);

        let parser = BlockParser::new();
        let result = parser.parse(categorized_block);

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_text());
    }

    #[test]
    fn should_parse_heading_block() {
        let src = "# This is a heading";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 20));
        let categorized_block = CategorizedBlock::new(BlockKind::Heading, src.to_string(), span);

        let parser = BlockParser::new();
        let result = parser.parse(categorized_block);

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_heading());
    }

    #[test]
    fn should_parse_list_block() {
        let src = "- This is a list item";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 22));
        let categorized_block = CategorizedBlock::new(BlockKind::List, src.to_string(), span);

        let parser = BlockParser::new();
        let result = parser.parse(categorized_block);

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_list());
    }
}
