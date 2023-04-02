use crate::parser::block::text::{TextBlock, Tree};
use crate::parser::block::ParsedBlockKind::Text;
use crate::parser::text::tokenizer::Tokenizer;
use crate::parser::{ParseResult, ParsedBlock};
use crate::util::SourceSpan;

mod token;
mod tokenizer;

pub(crate) struct TextParser {
    tokenizer: Tokenizer,
    tree: Tree,
}

impl TextParser {
    pub fn new(src: String, span: SourceSpan) -> Self {
        Self {
            tokenizer: Tokenizer::new(src, span.clone()),
            tree: Tree::new(span),
        }
    }

    pub fn parse(self) -> ParseResult<ParsedBlock> {
        for token in self.tokenizer {
            println!("{:?}", token); // TODO Map tokens to tree
        }

        let span = self.tree.root().span().clone();
        Ok(ParsedBlock::new(Text(TextBlock::new(self.tree)), span))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::block::ParsedBlockKind;
    use crate::util::SourcePosition;
    use crate::util::SourceSpan;

    use super::*;

    #[test]
    fn should_parse_trivial_text_block() {
        let src = "This is a paragraph.";
        let span = SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len()));

        let mut parser = TextParser::new(src.to_string(), span.clone());
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

        // TODO
    }
}
