use crate::parser::block::code::{CodeBlock, LanguageIdentifier};
use crate::parser::block::ParsedBlockKind;
use crate::parser::{ParseError, ParseResult, ParsedBlock};
use crate::util::SourceSpan;

pub(crate) struct CodeParser {
    src: String,
    span: SourceSpan,
}

struct CodeBlockHeader {
    offset: usize,
    language_identifier: Option<LanguageIdentifier>,
}

struct CodeBlockFooter {
    offset: usize,
}

impl CodeParser {
    pub fn new(src: String, span: SourceSpan) -> Self {
        Self { src, span }
    }

    pub fn parse(self) -> ParseResult<ParsedBlock> {
        let header = self.find_header()?;
        let footer = self.find_footer()?;

        println!("header.offset: {}", header.offset);
        println!("footer.offset: {}", footer.offset);

        let code_src = self.src[header.offset..footer.offset].trim().to_string();

        Ok(ParsedBlock::new(
            ParsedBlockKind::Code(CodeBlock::new(header.language_identifier, code_src)),
            self.span,
        ))
    }

    fn find_header(&self) -> ParseResult<CodeBlockHeader> {
        let mut offset = 0;

        let trimmed_src = self.src.trim_start();
        offset += self.src.len() - trimmed_src.len();

        if trimmed_src.starts_with("```") {
            offset += 3;

            let mut language_identifier = String::new();
            for c in trimmed_src[3..].chars() {
                match c {
                    ' ' | '\t' | '\n' => break,
                    '`' => {
                        language_identifier.clear();
                        break;
                    }
                    _ => language_identifier.push(c),
                };
            }

            return Ok(CodeBlockHeader {
                offset: offset + language_identifier.len(),
                language_identifier: if language_identifier.is_empty() {
                    None
                } else {
                    Some(language_identifier)
                },
            });
        }

        Err(ParseError {
            message: "Code block must be started with '```'".to_string(),
            source_position: self.span.start.clone(),
        })
    }

    fn find_footer(&self) -> ParseResult<CodeBlockFooter> {
        let trimmed_src = self.src.trim_end();
        let trimmed_src_len = trimmed_src.len();

        let mut offset = trimmed_src_len;

        if self.src.ends_with("```") {
            offset -= 3;

            return Ok(CodeBlockFooter { offset });
        }

        Err(ParseError {
            message: "Code block must be ended with '```'".to_string(),
            source_position: self.span.end.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::util::SourcePosition;

    use super::*;

    #[test]
    fn parse_code_block_without_language_identifier() {
        let src = "```
console.log('Hello World');
```";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 4));
        let parser = CodeParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_code());

        let code_block = if let ParsedBlockKind::Code(code_block) = parsed_block.into_kind() {
            code_block
        } else {
            panic!("Expected code block");
        };

        assert!(code_block.language().is_none());

        let code_src = code_block.src();
        assert_eq!(code_src, "console.log('Hello World');");
    }

    #[test]
    fn parse_code_block_with_language_identifier() {
        let src = "```js
console.log('Hello World');
```";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 4));
        let parser = CodeParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_code());

        let code_block = if let ParsedBlockKind::Code(code_block) = parsed_block.into_kind() {
            code_block
        } else {
            panic!("Expected code block");
        };

        assert_eq!(*code_block.language(), Some("js".to_string()));

        let code_src = code_block.src();
        assert_eq!(code_src, "console.log('Hello World');");
    }

    #[test]
    fn parse_multi_line_code_block() {
        let src = "```md
# Title

This is a simple paragraph.

I want it formatted in **bold** and as inline `code`.
```";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(7, 4));
        let parser = CodeParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_code());

        let code_block = if let ParsedBlockKind::Code(code_block) = parsed_block.into_kind() {
            code_block
        } else {
            panic!("Expected code block");
        };

        assert_eq!(*code_block.language(), Some("md".to_string()));

        let code_src = code_block.src();
        assert_eq!(
            code_src,
            "# Title

This is a simple paragraph.

I want it formatted in **bold** and as inline `code`."
        );
    }

    #[test]
    fn parse_weird_header_and_footer() {
        let src = "```md # Title

This is a simple paragraph.

I want it formatted in **bold** and as inline `code`.```";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(5, 57));
        let parser = CodeParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_code());

        let code_block = if let ParsedBlockKind::Code(code_block) = parsed_block.into_kind() {
            code_block
        } else {
            panic!("Expected code block");
        };

        assert_eq!(*code_block.language(), Some("md".to_string()));

        let code_src = code_block.src();
        assert_eq!(
            code_src,
            "# Title

This is a simple paragraph.

I want it formatted in **bold** and as inline `code`."
        );
    }

    #[test]
    fn fail_on_missing_footer() {
        let src = "```md # Title

This is a simple paragraph.

I want it formatted in **bold** and as inline `code`.";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(5, 54));
        let parser = CodeParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Code block must be ended with '```'"
        );
    }

    #[test]
    fn fail_on_incorrect_footer() {
        let src = "```md # Title

This is a simple paragraph.

I want it formatted in **bold** and as inline `code`.``";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(5, 56));
        let parser = CodeParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Code block must be ended with '```'"
        );
    }

    #[test]
    fn fail_on_missing_header() {
        let src = "# Title

This is a simple paragraph.

I want it formatted in **bold** and as inline `code`.
```";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(6, 4));
        let parser = CodeParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Code block must be started with '```'"
        );
    }

    #[test]
    fn fail_on_incorrect_header() {
        let src = "``# Title

This is a simple paragraph.

I want it formatted in **bold** and as inline `code`.
```";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(6, 4));
        let parser = CodeParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message,
            "Code block must be started with '```'"
        );
    }

    #[test]
    fn parse_empty_code_block() {
        let src = "``````";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 7));
        let parser = CodeParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_code());

        let code_block = if let ParsedBlockKind::Code(code_block) = parsed_block.into_kind() {
            code_block
        } else {
            panic!("Expected code block");
        };

        assert_eq!(*code_block.language(), None);

        let code_src = code_block.src();
        assert_eq!(code_src, "");
    }

    #[test]
    fn parse_minimal_code_block() {
        let src = "```a```";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 8));
        let parser = CodeParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_code());

        let code_block = if let ParsedBlockKind::Code(code_block) = parsed_block.into_kind() {
            code_block
        } else {
            panic!("Expected code block");
        };

        assert_eq!(*code_block.language(), None);

        let code_src = code_block.src();
        assert_eq!(code_src, "a");
    }
}
