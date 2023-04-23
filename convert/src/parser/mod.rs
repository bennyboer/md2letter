//! Parse various Markdown blocks.

pub(crate) use block::ParsedBlock;
pub(crate) use result::{ParseError, ParseResult};

use crate::categorizer::{BlockKind, CategorizedBlock};
use crate::parser::block::ParsedBlockKind;
use crate::parser::code::CodeParser;
use crate::parser::function::FunctionParser;
use crate::parser::heading::HeadingParser;
use crate::parser::image::ImageParser;
use crate::parser::list::ListParser;
use crate::parser::quote::QuoteParser;
use crate::parser::table::TableParser;
use crate::parser::text::TextParser;

mod block;
mod code;
mod function;
mod heading;
mod image;
mod list;
mod quote;
mod result;
mod table;
mod text;

pub(crate) struct BlockParser;

impl BlockParser {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn parse(&self, categorized_block: CategorizedBlock) -> ParseResult<ParsedBlock> {
        let (kind, src, span) = categorized_block.consume();

        match kind {
            BlockKind::Text => TextParser::new(src, span).parse(),
            BlockKind::Heading => HeadingParser::new(src, span).parse(),
            BlockKind::List => ListParser::new(src, span).parse(),
            BlockKind::HorizontalRule => {
                Ok(ParsedBlock::new(ParsedBlockKind::HorizontalRule, span))
            }
            BlockKind::Code => CodeParser::new(src, span).parse(),
            BlockKind::Table => TableParser::new(src, span).parse(),
            BlockKind::Image => ImageParser::new(src, span).parse(),
            BlockKind::Quote => QuoteParser::new(src, span).parse(),
            BlockKind::Function => FunctionParser::new(src, span).parse(),
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

    #[test]
    fn should_parse_horizontal_rule_block() {
        let src = "---";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 4));
        let categorized_block =
            CategorizedBlock::new(BlockKind::HorizontalRule, src.to_string(), span);

        let parser = BlockParser::new();
        let result = parser.parse(categorized_block);

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_horizontal_rule());
    }

    #[test]
    fn should_parse_code_block() {
        let src = "```js
console.log('Hello World');
```";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 4));
        let categorized_block = CategorizedBlock::new(BlockKind::Code, src.to_string(), span);

        let parser = BlockParser::new();
        let result = parser.parse(categorized_block);

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_code());
    }

    #[test]
    fn should_parse_table_block() {
        let src = "| Small | Table |
| ----- | ----- |
| 1     | 2     |
| 3     | 4     |";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(4, 19));
        let categorized_block = CategorizedBlock::new(BlockKind::Table, src.to_string(), span);

        let parser = BlockParser::new();
        let result = parser.parse(categorized_block);

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_table());
    }

    #[test]
    fn should_parse_image_block() {
        let src = "![alt text](image.jpg)";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 23));
        let categorized_block = CategorizedBlock::new(BlockKind::Image, src.to_string(), span);

        let parser = BlockParser::new();
        let result = parser.parse(categorized_block);

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_image());
    }

    #[test]
    fn should_parse_quote_block() {
        let src = "> This is a quote";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 18));
        let categorized_block = CategorizedBlock::new(BlockKind::Quote, src.to_string(), span);

        let parser = BlockParser::new();
        let result = parser.parse(categorized_block);

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_quote());
    }

    #[test]
    fn should_parse_function_block() {
        let src = "#TableOfContents";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 17));
        let categorized_block = CategorizedBlock::new(BlockKind::Function, src.to_string(), span);

        let parser = BlockParser::new();
        let result = parser.parse(categorized_block);

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_function());
    }
}
