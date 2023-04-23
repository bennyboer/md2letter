use crate::parser::{ParsedBlock, ParseError, ParseResult};
use crate::parser::block::ParsedBlockKind;
use crate::parser::block::quote::{QuoteBlock, QuoteNodeId, QuoteNodeKind, QuoteTree};
use crate::parser::text::TextParser;
use crate::util::{SourcePosition, SourceSpan};

pub(crate) struct QuoteParser {
    src: String,
    span: SourceSpan,
}

#[derive(Debug)]
struct IndentedQuoteLine {
    line: String,
    line_number: usize,
    indent: usize,
    offset: usize,
}

impl QuoteParser {
    pub fn new(src: String, span: SourceSpan) -> Self {
        Self { src, span }
    }

    pub fn parse(self) -> ParseResult<ParsedBlock> {
        let mut tree = QuoteTree::new();
        let indented_quote_lines = self.find_indented_quote_lines()?;

        let mut parent_node_ids = vec![tree.root().id()];
        let mut indents = Vec::new();
        let mut text_buffer = String::new();
        let mut counter = 0;
        let mut start_line_number = 0;
        let mut start_offset = 0;
        let total = indented_quote_lines.len();
        for indented_quote_line in indented_quote_lines {
            if indents.is_empty() {
                indents.push(indented_quote_line.indent);
                start_line_number = indented_quote_line.line_number;
                start_offset = indented_quote_line.offset;
            }

            let mut current_parent_id = *parent_node_ids.last().unwrap();
            let current_indent = *indents.last().unwrap();

            let is_same_indent = current_indent == indented_quote_line.indent;
            if is_same_indent {
                if !text_buffer.is_empty() {
                    text_buffer.push(' ');
                }
                text_buffer.push_str(&indented_quote_line.line);
            } else if current_indent < indented_quote_line.indent {
                self.consume_text_buffer_into_node(
                    &mut tree,
                    &mut text_buffer,
                    &indented_quote_line,
                    current_parent_id,
                    start_line_number,
                    start_offset,
                )?;

                parent_node_ids.push(tree.register_node(current_parent_id, QuoteNodeKind::Parent));
                indents.push(indented_quote_line.indent);
                current_parent_id = *parent_node_ids.last().unwrap();

                text_buffer.push_str(&indented_quote_line.line);
                start_line_number = indented_quote_line.line_number;
                start_offset = indented_quote_line.offset;
            } else {
                self.consume_text_buffer_into_node(
                    &mut tree,
                    &mut text_buffer,
                    &indented_quote_line,
                    current_parent_id,
                    start_line_number,
                    start_offset,
                )?;

                while !indents.is_empty() && *indents.last().unwrap() > indented_quote_line.indent {
                    indents.pop();
                    parent_node_ids.pop();
                }
                current_parent_id = *parent_node_ids.last().unwrap();

                text_buffer.push_str(&indented_quote_line.line);
                start_line_number = indented_quote_line.line_number;
                start_offset = indented_quote_line.offset;
            }

            let is_last = counter == total - 1;
            if is_last {
                self.consume_text_buffer_into_node(
                    &mut tree,
                    &mut text_buffer,
                    &indented_quote_line,
                    current_parent_id,
                    start_line_number,
                    start_offset,
                )?;
            }

            counter += 1;
        }

        Ok(ParsedBlock::new(
            ParsedBlockKind::Quote(QuoteBlock::new(tree)),
            self.span,
        ))
    }

    fn consume_text_buffer_into_node(
        &self,
        tree: &mut QuoteTree,
        text_buffer: &mut String,
        indented_quote_line: &IndentedQuoteLine,
        current_parent_id: QuoteNodeId,
        start_line_number: usize,
        start_offset: usize,
    ) -> ParseResult<()> {
        let text_parser = TextParser::new(
            text_buffer.clone(),
            SourceSpan::new(
                SourcePosition::new(start_line_number, start_offset + 1),
                SourcePosition::new(
                    indented_quote_line.line_number,
                    indented_quote_line.offset + 1 + indented_quote_line.line.len(),
                ),
            ),
        );
        let parsed_block = text_parser.parse()?;
        let text_block = if let ParsedBlockKind::Text(text_block) = parsed_block.into_kind() {
            text_block
        } else {
            unreachable!()
        };
        let text_tree = text_block.into_tree();
        tree.register_node(current_parent_id, QuoteNodeKind::Leaf { text_tree });

        text_buffer.clear();

        Ok(())
    }

    fn find_indented_quote_lines(&self) -> ParseResult<Vec<IndentedQuoteLine>> {
        let mut result = Vec::new();

        let mut line_number = self.span.start.line;
        for line in self.src.lines() {
            let mut indent = 0;
            let mut offset = 0;
            for c in line.chars() {
                match c {
                    '\t' | ' ' => {}
                    '>' => {
                        indent += 1;
                    }
                    _ => {
                        if indent == 0 {
                            return Err(ParseError {
                                message: format!(
                                    "Found no quote line start character '>' in line {}",
                                    line_number
                                ),
                                source_position: SourcePosition::new(line_number, offset + 1),
                            });
                        } else {
                            break;
                        }
                    }
                }

                offset += 1;
            }

            result.push(IndentedQuoteLine {
                line: line[offset..].to_string(),
                line_number,
                indent,
                offset,
            });

            line_number += 1;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use crate::util::SourcePosition;

    use super::*;

    #[test]
    fn should_parse_trivial_quote_block() {
        let src = "> This is a quote!";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 19));
        let parser = QuoteParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_quote());

        let quote_block = if let ParsedBlockKind::Quote(quote_block) = parsed_block.into_kind() {
            quote_block
        } else {
            panic!("Expected quote block");
        };

        let quote_tree = quote_block.into_tree();

        assert_eq!(
            format!("{}", quote_tree),
            "- [Parent]
  - [Leaf]
    - [Text](This is a quote!)
"
        );
    }

    #[test]
    fn should_parse_multilevel_quote_block_1() {
        let src = "> This is
> a quote!
>> And this as well!
> Another first-level quote.";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(4, 29));
        let parser = QuoteParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_quote());

        let quote_block = if let ParsedBlockKind::Quote(quote_block) = parsed_block.into_kind() {
            quote_block
        } else {
            panic!("Expected quote block");
        };

        let quote_tree = quote_block.into_tree();

        assert_eq!(
            format!("{}", quote_tree),
            "- [Parent]
  - [Leaf]
    - [Text](This is a quote!)
  - [Parent]
    - [Leaf]
      - [Text](And this as well!)
  - [Leaf]
    - [Text](Another first-level quote.)
"
        );
    }

    #[test]
    fn should_parse_multilevel_quote_block_2() {
        let src = "> This is
> a quote!
>> And **this** as well!
>> > > Wow
>>>> Hey hey
> Another first-level quote.";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(6, 29));
        let parser = QuoteParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_quote());

        let quote_block = if let ParsedBlockKind::Quote(quote_block) = parsed_block.into_kind() {
            quote_block
        } else {
            panic!("Expected quote block");
        };

        let quote_tree = quote_block.into_tree();

        assert_eq!(
            format!("{}", quote_tree),
            "- [Parent]
  - [Leaf]
    - [Text](This is a quote!)
  - [Parent]
    - [Leaf]
      - [Text](And )
      - [Bold]
        - [Text](this)
      - [Text]( as well!)
    - [Parent]
      - [Leaf]
        - [Text](Wow Hey hey)
  - [Leaf]
    - [Text](Another first-level quote.)
"
        );
    }
}
