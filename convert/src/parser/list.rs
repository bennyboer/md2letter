use crate::parser::block::list::{ListBlock, ListNodeKind, ListNodeStyle, ListTree};
use crate::parser::block::ParsedBlockKind;
use crate::parser::text::TextParser;
use crate::parser::{ParseError, ParseResult, ParsedBlock};
use crate::util::{SourcePosition, SourceSpan};

pub(crate) struct ListParser {
    src: String,
    span: SourceSpan,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Indent {
    Zero,
    Tab(usize),
    Space(usize),
}

#[derive(Debug)]
struct ItemInSource {
    indent: Indent,
    _symbol: String,
    is_ordered: bool,
    content: String,
    span: SourceSpan,
}

struct IsStartOfNewLineResult {
    is_start_of_new_item: bool,
    indent: Indent,
    symbol: String,
    is_ordered: bool,
}

impl Indent {
    fn count(&self) -> usize {
        match self {
            Indent::Zero => 0,
            Indent::Tab(count) => *count,
            Indent::Space(count) => *count,
        }
    }
}

impl ListParser {
    pub fn new(src: String, span: SourceSpan) -> Self {
        Self { src, span }
    }

    pub fn parse(self) -> ParseResult<ParsedBlock> {
        let items = self.find_items_in_src()?;
        let mut tree = ListTree::new();
        let mut parent_node_id_stack = vec![tree.root().id()];
        let mut required_indents: Vec<Indent> = vec![Indent::Zero];

        for item in items {
            let parent_node_id = *parent_node_id_stack.last().unwrap();
            let parent_node_level = parent_node_id_stack.len();
            let required_indent_for_same_level =
                *required_indents.get(parent_node_level - 1).unwrap();

            let list_node_style = if item.is_ordered {
                ListNodeStyle::Ordered
            } else {
                ListNodeStyle::Unordered
            };
            let text_parser = TextParser::new(item.content, item.span);
            let text_block = text_parser.parse()?;
            let text_tree = if let ParsedBlockKind::Text(text_block) = text_block.into_kind() {
                text_block.into_tree()
            } else {
                unreachable!()
            };

            if item.indent == required_indent_for_same_level {
                tree.register_node(
                    parent_node_id,
                    ListNodeKind::Leaf { text_tree },
                    list_node_style,
                );
            } else if item.indent.count() > required_indent_for_same_level.count() {
                let new_parent_node_id =
                    tree.register_node(parent_node_id, ListNodeKind::Parent, list_node_style);
                parent_node_id_stack.push(new_parent_node_id);
                required_indents.push(item.indent);

                tree.register_node(
                    new_parent_node_id,
                    ListNodeKind::Leaf { text_tree },
                    list_node_style,
                );
            } else {
                required_indents.pop();
                parent_node_id_stack.pop();
                let new_parent_node_id = *parent_node_id_stack.last().unwrap();

                tree.register_node(
                    new_parent_node_id,
                    ListNodeKind::Leaf { text_tree },
                    list_node_style,
                );
            }
        }

        Ok(ParsedBlock::new(
            ParsedBlockKind::List(ListBlock::new(tree)),
            self.span,
        ))
    }

    fn find_items_in_src(&self) -> ParseResult<Vec<ItemInSource>> {
        let src = self.src.as_str();
        let mut items = Vec::new();

        for (index, line) in src.lines().enumerate() {
            let line_number = self.span.start.line + index;
            let IsStartOfNewLineResult {
                is_start_of_new_item,
                indent,
                symbol,
                is_ordered,
            } = self.is_start_of_new_item(line, line_number)?;

            if is_start_of_new_item {
                let indent_count = indent.count();
                let symbol_length = symbol.len();

                items.push(ItemInSource {
                    indent,
                    _symbol: symbol,
                    is_ordered,
                    content: line[indent_count + symbol_length + 1..].to_owned(),
                    span: SourceSpan::new(
                        SourcePosition::new(line_number, 1),
                        SourcePosition::new(line_number, line.len() + 1),
                    ),
                });
            } else {
                let last_item = items.last_mut().unwrap();
                last_item.content.push_str(line);
                last_item.span.end = SourcePosition::new(line_number, line.len() + 1);
            }
        }

        Ok(items)
    }

    fn is_start_of_new_item(
        &self,
        line: &str,
        line_number: usize,
    ) -> ParseResult<IsStartOfNewLineResult> {
        let mut indent = Indent::Zero;

        for (index, c) in line.chars().enumerate() {
            match c {
                '\t' => match indent {
                    Indent::Zero => indent = Indent::Tab(1),
                    Indent::Tab(count) => {
                        indent = Indent::Tab(count + 1);
                    }
                    Indent::Space(_) => {
                        return Err(ParseError {
                                message: "Mixed tab and space in list item indentation. Started indenting with tab and then encountered space.".to_string(),
                                source_position: SourcePosition::new(line_number, 1),
                            });
                    }
                },
                ' ' => match indent {
                    Indent::Zero => indent = Indent::Space(1),
                    Indent::Space(count) => {
                        indent = Indent::Space(count + 1);
                    }
                    Indent::Tab(_) => {
                        return Err(ParseError {
                                message: "Mixed tab and space in list item indentation. Started indenting with tab and then encountered space.".to_string(),
                                source_position: SourcePosition::new(line_number, 1),
                            });
                    }
                },
                _ => {
                    let is_valid_unordered_symbol = c == '-' || c == '*' || c == '+';
                    let is_valid_ordered_symbol =
                        c.is_digit(10) && line.chars().nth(index + 1) == Some('.');
                    let symbol = if is_valid_unordered_symbol {
                        Some(c.to_string())
                    } else if is_valid_ordered_symbol {
                        Some(line[index..index + 2].to_string())
                    } else {
                        None
                    };

                    let is_followed_by_space = symbol
                        .as_ref()
                        .map(|s| line.chars().nth(index + s.len()) == Some(' '))
                        .unwrap_or(false);

                    return Ok(IsStartOfNewLineResult {
                        is_start_of_new_item: symbol.is_some() && is_followed_by_space,
                        indent,
                        symbol: symbol.unwrap_or("".to_string()),
                        is_ordered: is_valid_ordered_symbol,
                    });
                }
            }
        }

        Ok(IsStartOfNewLineResult {
            is_start_of_new_item: false,
            indent,
            symbol: "".to_string(),
            is_ordered: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::block::ParsedBlockKind;
    use crate::parser::list::ListParser;
    use crate::util::{SourcePosition, SourceSpan};

    #[test]
    fn should_parse_simple_unordered_list() {
        let src = "- Item 1
- Item 2
- Item 3";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 9));
        let parser = ListParser::new(src.to_string(), span);

        let result = parser.parse();

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_list());

        let list_block = if let ParsedBlockKind::List(list_block) = parsed_block.into_kind() {
            list_block
        } else {
            panic!("Expected list block");
        };

        let tree = list_block.into_tree();
        assert_eq!(
            format!("{}", tree),
            "- [Parent]
  - unordered [Item]
    - [Text](Item 1)
  - unordered [Item]
    - [Text](Item 2)
  - unordered [Item]
    - [Text](Item 3)
"
        );
    }

    #[test]
    fn should_parse_simple_ordered_list() {
        let src = "1. Item 1
2. Item 2
3. Item 3";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 9));
        let parser = ListParser::new(src.to_string(), span);

        let result = parser.parse();

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_list());

        let list_block = if let ParsedBlockKind::List(list_block) = parsed_block.into_kind() {
            list_block
        } else {
            panic!("Expected list block");
        };

        let tree = list_block.into_tree();
        assert_eq!(
            format!("{}", tree),
            "- [Parent]
  - ordered [Item]
    - [Text](Item 1)
  - ordered [Item]
    - [Text](Item 2)
  - ordered [Item]
    - [Text](Item 3)
"
        );
    }

    #[test]
    fn should_parse_nested_list() {
        let src = r#"- Item 1
  1. Item 1.1
  2. Item 1.2
- Item 2
- Item 3"#;
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 9));
        let parser = ListParser::new(src.to_string(), span);

        let result = parser.parse();

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_list());

        let list_block = if let ParsedBlockKind::List(list_block) = parsed_block.into_kind() {
            list_block
        } else {
            panic!("Expected list block");
        };

        let tree = list_block.into_tree();
        assert_eq!(
            format!("{}", tree),
            "- [Parent]
  - unordered [Item]
    - [Text](Item 1)
  - [Parent]
    - ordered [Item]
      - [Text](Item 1.1)
    - ordered [Item]
      - [Text](Item 1.2)
  - unordered [Item]
    - [Text](Item 2)
  - unordered [Item]
    - [Text](Item 3)
"
        );
    }

    #[test]
    fn should_parse_crazy_list() {
        let src = r#"- Item **1**
  1. Item 1.1
    * Item 1.1.1
    * Item 1.1.2
  2. Item 1.2
    + Item 1.2.1
    + Item 1.2.2
      1. Another Item 1
      1. Another Item 2
    + Item 1.2.3
- Item `2`
- Item 3"#;
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 9));
        let parser = ListParser::new(src.to_string(), span);

        let result = parser.parse();

        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_list());

        let list_block = if let ParsedBlockKind::List(list_block) = parsed_block.into_kind() {
            list_block
        } else {
            panic!("Expected list block");
        };

        let tree = list_block.into_tree();
        assert_eq!(
            format!("{}", tree),
            "- [Parent]
  - unordered [Item]
    - [Text](Item )
    - [Bold]
      - [Text](1)
  - [Parent]
    - ordered [Item]
      - [Text](Item 1.1)
    - [Parent]
      - unordered [Item]
        - [Text](Item 1.1.1)
      - unordered [Item]
        - [Text](Item 1.1.2)
    - ordered [Item]
      - [Text](Item 1.2)
    - [Parent]
      - unordered [Item]
        - [Text](Item 1.2.1)
      - unordered [Item]
        - [Text](Item 1.2.2)
      - [Parent]
        - ordered [Item]
          - [Text](Another Item 1)
        - ordered [Item]
          - [Text](Another Item 2)
      - unordered [Item]
        - [Text](Item 1.2.3)
    - unordered [Item]
      - [Text](Item )
      - [Code]
        - [Text](2)
  - unordered [Item]
    - [Text](Item 3)
"
        );
    }
}
