use std::collections::HashMap;

use crate::parser::{ParsedBlock, ParseError, ParseResult};
use crate::parser::block::function::FunctionBlock;
use crate::parser::block::ParsedBlockKind;
use crate::util::SourceSpan;

pub(crate) struct FunctionParser {
    src: String,
    span: SourceSpan,
}

impl FunctionParser {
    pub fn new(src: String, span: SourceSpan) -> Self {
        Self { src, span }
    }

    pub fn parse(self) -> ParseResult<ParsedBlock> {
        let src = self.src.trim();
        let mut name = String::new();
        let mut offset = 0;
        for c in src.chars() {
            match c {
                '#' => {}
                ' ' | '\t' => {
                    return Err(ParseError {
                        message: "Unexpected whitespace in function name".to_owned(),
                        source_position: self.span.start.clone(),
                    });
                }
                '(' => {
                    break;
                }
                _ => {
                    name.push(c);
                }
            }

            offset += 1;
        }

        if name.is_empty() {
            return Err(ParseError {
                message: "Function name is empty".to_owned(),
                source_position: self.span.start.clone(),
            });
        }

        let mut parameters = HashMap::new();
        let src = &src[offset..];
        if src.chars().nth(0) == Some('(') {
            if src.chars().last() != Some(')') {
                return Err(ParseError {
                    message: "Expected closing parenthesis for function parameters".to_owned(),
                    source_position: self.span.start.clone(),
                });
            }

            let parameters_str = src[1..src.len() - 1].to_owned();
            if !parameters_str.is_empty() {
                let entries = parameters_str.split(',');
                for entry in entries {
                    let mut parts = entry.split(':');
                    let key = parts.next().unwrap().trim().to_owned();
                    let value = parts.next().unwrap().trim().to_owned();
                    parameters.insert(key, value);
                }
            }
        }

        Ok(ParsedBlock::new(
            ParsedBlockKind::Function(FunctionBlock::new(name, parameters)),
            self.span,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::util::SourcePosition;

    use super::*;

    #[test]
    fn should_parse_function_block_without_params_1() {
        let src = "#TableOfContents";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 17));
        let parser = FunctionParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_function());

        let function_block =
            if let ParsedBlockKind::Function(function_block) = parsed_block.into_kind() {
                function_block
            } else {
                panic!("Expected function block");
            };

        assert_eq!(function_block.name(), "TableOfContents");
        assert!(function_block.parameters().is_empty());
    }

    #[test]
    fn should_parse_function_block_without_params_2() {
        let src = "#TableOfContents()";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 17));
        let parser = FunctionParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_function());

        let function_block =
            if let ParsedBlockKind::Function(function_block) = parsed_block.into_kind() {
                function_block
            } else {
                panic!("Expected function block");
            };

        assert_eq!(function_block.name(), "TableOfContents");
        assert!(function_block.parameters().is_empty());
    }

    #[test]
    fn should_parse_function_block_with_params() {
        let src = "#Image(
  width: 100px,
  height: 200px,
  src: image.jpg
)";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(5, 2));
        let parser = FunctionParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_function());

        let function_block =
            if let ParsedBlockKind::Function(function_block) = parsed_block.into_kind() {
                function_block
            } else {
                panic!("Expected function block");
            };

        assert_eq!(function_block.name(), "Image");

        let parameters = function_block.parameters();
        assert_eq!(parameters.len(), 3);
        assert_eq!(parameters.get("width").unwrap(), "100px");
        assert_eq!(parameters.get("height").unwrap(), "200px");
        assert_eq!(parameters.get("src").unwrap(), "image.jpg");
    }
}
