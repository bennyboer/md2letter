use crate::parser::block::image::ImageBlock;
use crate::parser::block::ParsedBlockKind;
use crate::parser::text::TextParser;
use crate::parser::{ParseResult, ParsedBlock};
use crate::util::{SourcePosition, SourceSpan};

pub(crate) struct ImageParser {
    src: String,
    span: SourceSpan,
}

impl ImageParser {
    pub fn new(src: String, span: SourceSpan) -> Self {
        Self { src, span }
    }

    pub fn parse(self) -> ParseResult<ParsedBlock> {
        let src = self.src.trim();
        let mut offset = self.src.len() - src.len();
        let src = &src[2..];

        // Find closing ']'
        let mut closing_bracket_offset = 0;
        let mut ignore_next_closing_brackets = 0;
        for (i, c) in src.char_indices() {
            match c {
                '[' => ignore_next_closing_brackets += 1,
                ']' => {
                    if ignore_next_closing_brackets > 0 {
                        ignore_next_closing_brackets -= 1;
                    } else {
                        closing_bracket_offset = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        let text_src = src[..closing_bracket_offset].trim();
        let text_parser = TextParser::new(
            text_src.to_string(),
            SourceSpan::new(
                SourcePosition::new(self.span.start.line, offset + 2),
                SourcePosition::new(self.span.start.line, offset + 2 + closing_bracket_offset),
            ),
        );
        let text_block = text_parser.parse()?;
        let text_block = if let ParsedBlockKind::Text(text_block) = text_block.into_kind() {
            text_block
        } else {
            unreachable!()
        };
        let text_tree = text_block.into_tree();

        // Find image src
        let src = &src[closing_bracket_offset + 2..];
        let mut image_src = String::new();
        for c in src.chars() {
            match c {
                ')' => break,
                _ => image_src.push(c),
            }
        }

        Ok(ParsedBlock::new(
            ParsedBlockKind::Image(ImageBlock::new(text_tree, image_src)),
            self.span,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_image() {
        let src = "![Label](image.jpg)";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 20));
        let parser = ImageParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_image());

        let image_block = if let ParsedBlockKind::Image(image_block) = parsed_block.into_kind() {
            image_block
        } else {
            panic!("Expected image block");
        };

        assert_eq!(image_block.src(), "image.jpg");

        let text_tree = image_block.text_tree();
        assert_eq!(
            format!("{}", text_tree),
            "- [Root]
  - [Text](Label)
"
        );
    }

    #[test]
    fn should_parse_image_with_formatting_in_label() {
        let src = "![Label **with** formatting](image.jpg)";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 20));
        let parser = ImageParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_image());

        let image_block = if let ParsedBlockKind::Image(image_block) = parsed_block.into_kind() {
            image_block
        } else {
            panic!("Expected image block");
        };

        assert_eq!(image_block.src(), "image.jpg");

        let text_tree = image_block.text_tree();
        assert_eq!(
            format!("{}", text_tree),
            "- [Root]
  - [Text](Label )
  - [Bold]
    - [Text](with)
  - [Text]( formatting)
"
        );
    }
}
