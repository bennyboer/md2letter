use TokenKind::{
    BoldEnd, BoldStart, CodeEnd, CodeStart, Error, Function, Image, ItalicEnd, ItalicStart, Link,
    Text,
};

pub(crate) use crate::parser::block::text::{TextBlock, TextNodeKind, TextTree};
use crate::parser::block::ParsedBlockKind;
use crate::parser::text::token::TokenKind;
use crate::parser::text::tokenizer::Tokenizer;
use crate::parser::{ParseError, ParseResult, ParsedBlock};
use crate::util::SourceSpan;

mod token;
mod tokenizer;

pub(crate) struct TextParser {
    tokenizer: Tokenizer,
    tree: TextTree,
}

impl TextParser {
    pub fn new(src: String, span: SourceSpan) -> Self {
        Self {
            tokenizer: Tokenizer::new(src, span.clone()),
            tree: TextTree::new(span),
        }
    }

    pub fn parse(mut self) -> ParseResult<ParsedBlock> {
        let mut parent_node_id_stack = vec![self.tree.root().id()];

        for token in self.tokenizer {
            let parent_node_id = *parent_node_id_stack.last().unwrap();
            let span = token.span().clone();

            match token.kind() {
                Error {
                    message,
                    source_position,
                } => {
                    return Err(ParseError {
                        message: message.clone(),
                        source_position: source_position.clone(),
                    });
                }
                Text(s) => {
                    self.tree.register_node(
                        parent_node_id,
                        TextNodeKind::Text { src: s.clone() },
                        span,
                    );
                }
                Link { label, target } => {
                    let node_id = self.tree.register_node(
                        parent_node_id,
                        TextNodeKind::Link {
                            target: target.clone(),
                        },
                        span.clone(),
                    );
                    self.tree.register_node(
                        node_id,
                        TextNodeKind::Text { src: label.clone() },
                        span,
                    );
                }
                Image { label, src } => {
                    let node_id = self.tree.register_node(
                        parent_node_id,
                        TextNodeKind::Image { src: src.clone() },
                        span.clone(),
                    );
                    self.tree.register_node(
                        node_id,
                        TextNodeKind::Text { src: label.clone() },
                        span,
                    );
                }
                Function { name, parameters } => {
                    self.tree.register_node(
                        parent_node_id,
                        TextNodeKind::Function {
                            name: name.clone(),
                            parameters: parameters.clone(),
                        },
                        span,
                    );
                }
                BoldStart | ItalicStart | CodeStart => {
                    let node_kind = match token.kind() {
                        BoldStart => TextNodeKind::Bold,
                        ItalicStart => TextNodeKind::Italic,
                        CodeStart => TextNodeKind::Code,
                        _ => unreachable!(),
                    };

                    let node_id = self.tree.register_node(parent_node_id, node_kind, span);

                    parent_node_id_stack.push(node_id);
                }
                BoldEnd | ItalicEnd | CodeEnd => {
                    parent_node_id_stack.pop();
                }
            }
        }

        let span = self.tree.root().span().clone();
        Ok(ParsedBlock::new(
            ParsedBlockKind::Text(TextBlock::new(self.tree)),
            span,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::util::SourcePosition;
    use crate::util::SourceSpan;

    use super::*;

    #[test]
    fn should_parse_trivial_text_block() {
        let src = "This is a paragraph.";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 21));

        let parser = TextParser::new(src.to_string(), span.clone());
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

        assert_eq!(
            format!("{}", tree),
            "- [Root]
  - [Text](This is a paragraph.)
"
        );
    }

    #[test]
    fn should_parse_text_block_with_simple_formatting() {
        let src = "Column *A*";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 21));

        let parser = TextParser::new(src.to_string(), span.clone());
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

        assert_eq!(
            format!("{}", tree),
            "- [Root]
  - [Text](Column )
  - [Italic]
    - [Text](A)
"
        );
    }

    #[test]
    fn should_parse_text_block_with_formatting() {
        let src = "This is **bold** and this is *italic* and this is ***both***, while *this is **mixed** and this is not*.";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 105));

        let parser = TextParser::new(src.to_string(), span.clone());
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

        assert_eq!(
            format!("{}", tree),
            "- [Root]
  - [Text](This is )
  - [Bold]
    - [Text](bold)
  - [Text]( and this is )
  - [Italic]
    - [Text](italic)
  - [Text]( and this is )
  - [Italic]
    - [Bold]
      - [Text](both)
  - [Text](, while )
  - [Italic]
    - [Text](this is )
    - [Bold]
      - [Text](mixed)
    - [Text]( and this is not)
  - [Text](.)
"
        );
    }

    #[test]
    fn should_parse_text_block_with_link() {
        let src = "This is a **[link](https://example.com)**.";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 43));

        let parser = TextParser::new(src.to_string(), span.clone());
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

        assert_eq!(
            format!("{}", tree),
            "- [Root]
  - [Text](This is a )
  - [Bold]
    - [Link](https://example.com)
      - [Text](link)
  - [Text](.)
"
        );
    }

    #[test]
    fn should_parse_text_block_with_image() {
        let src = "This is a **![my-image](my-image.png)**.";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 43));

        let parser = TextParser::new(src.to_string(), span.clone());
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

        assert_eq!(
            format!("{}", tree),
            "- [Root]
  - [Text](This is a )
  - [Bold]
    - [Image](my-image.png)
      - [Text](my-image)
  - [Text](.)
"
        );
    }

    #[test]
    fn should_parse_text_block_with_function() {
        let src = "This is a function: #Image(
  src: my-image.png,
  width: 200px,
  height: 100px
).";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(5, 4));

        let parser = TextParser::new(src.to_string(), span.clone());
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

        assert_eq!(
            format!("{}", tree),
            "- [Root]
  - [Text](This is a function: )
  - [Function](Image, height: 100px, src: my-image.png, width: 200px)
  - [Text](.)
"
        );
    }
}
