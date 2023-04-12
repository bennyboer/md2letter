use crate::parser::block::heading::HeadingBlock;
use crate::parser::block::ParsedBlockKind;
use crate::parser::text::TextParser;
use crate::parser::{ParseResult, ParsedBlock};
use crate::util::SourceSpan;

pub(crate) struct HeadingParser {
    src: String,
    span: SourceSpan,
}

struct FindHeadingLevelResult {
    heading_level: usize,
    offset: usize,
}

impl HeadingParser {
    pub fn new(src: String, span: SourceSpan) -> Self {
        Self { src, span }
    }

    pub fn parse(self) -> ParseResult<ParsedBlock> {
        let FindHeadingLevelResult {
            heading_level,
            offset,
        } = self.find_heading_level();

        let rest_str = self.src[offset..].to_owned();
        let text_parser = TextParser::new(rest_str, self.span.clone());
        let parsed_block = text_parser.parse()?;
        let text_block = if let ParsedBlockKind::Text(parsed_block) = parsed_block.into_kind() {
            parsed_block
        } else {
            unreachable!()
        };
        let text_tree = text_block.into_tree();

        Ok(ParsedBlock::new(
            ParsedBlockKind::Heading(HeadingBlock::new(heading_level, text_tree)),
            self.span,
        ))
    }

    fn find_heading_level(&self) -> FindHeadingLevelResult {
        let mut heading_level = 0;

        for c in self.src.chars() {
            if c == '#' {
                heading_level += 1;
            } else {
                break;
            }
        }

        FindHeadingLevelResult {
            heading_level,
            offset: heading_level + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::SourcePosition;

    use super::*;

    #[test]
    fn should_parse_first_level_heading() {
        let src = "# This is a heading";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 20));

        let parser = HeadingParser::new(src.to_string(), span.clone());
        let result = parser.parse();

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_heading());

        assert_eq!(span, parsed_block.span().clone());

        let heading_block = if let ParsedBlockKind::Heading(b) = parsed_block.into_kind() {
            b
        } else {
            panic!("Expected heading block");
        };

        assert_eq!(heading_block.level(), 1);

        assert_eq!(
            format!("{}", heading_block.text_tree()),
            "- [Root]
  - [Text](This is a heading)
"
        );
    }

    #[test]
    fn should_parse_multi_level_heading() {
        let src = "### This is third level heading";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 20));

        let parser = HeadingParser::new(src.to_string(), span.clone());
        let result = parser.parse();

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_heading());

        assert_eq!(span, parsed_block.span().clone());

        let heading_block = if let ParsedBlockKind::Heading(b) = parsed_block.into_kind() {
            b
        } else {
            panic!("Expected heading block");
        };

        assert_eq!(heading_block.level(), 3);

        assert_eq!(
            format!("{}", heading_block.text_tree()),
            "- [Root]
  - [Text](This is third level heading)
"
        );
    }

    #[test]
    fn should_parse_crazy_heading() {
        let src = "########## This is a **crazy** [heading](https://example.com)
with `multiple` *lines*!!!";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(2, 27));

        let parser = HeadingParser::new(src.to_string(), span.clone());
        let result = parser.parse();

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_heading());

        assert_eq!(span, parsed_block.span().clone());

        let heading_block = if let ParsedBlockKind::Heading(b) = parsed_block.into_kind() {
            b
        } else {
            panic!("Expected heading block");
        };

        assert_eq!(heading_block.level(), 10);

        assert_eq!(
            format!("{}", heading_block.text_tree()),
            "- [Root]
  - [Text](This is a )
  - [Bold]
    - [Text](crazy)
  - [Text]( )
  - [Link](https://example.com)
    - [Text](heading)
  - [Text]( with )
  - [Code]
    - [Text](multiple)
  - [Text]( )
  - [Italic]
    - [Text](lines)
  - [Text](!!!)
"
        );
    }
}
