use crate::categorizer::CategorizedBlock;
use crate::parser::block::text::Tree;
use crate::parser::{ParseResult, ParsedBlock};

pub(crate) struct TextParser {
    tree: Tree,
}

impl TextParser {
    pub fn new(block: CategorizedBlock) -> Self {
        Self {
            tree: Tree::new(block.span().clone()),
        }
    }

    pub fn parse(&mut self) -> ParseResult<ParsedBlock> {
        todo!("Parse text block");
    }
}

#[cfg(test)]
mod tests {
    use BlockKind::Text;

    use crate::categorizer::BlockKind;
    use crate::parser::block::ParsedBlockKind;
    use crate::util::SourcePosition;
    use crate::util::SourceSpan;

    use super::*;

    #[test]
    fn should_parse_trivial_text_block() {
        let src = "This is a paragraph.";
        let span = SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len()));
        let block = CategorizedBlock::new(Text, src.to_string(), span.clone());

        let mut parser = TextParser::new(block);
        let result = parser.parse();

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_text());

        assert_eq!(span, parsed_block.span().clone());

        let tree = if let ParsedBlockKind::Text(b) = parsed_block.kind() {
            b.tree()
        } else {
            panic!("Expected text block");
        };

        assert_eq!(1, tree.root().children().len());
        assert_eq!(src, tree.root().children()[0].text());
    }
}
