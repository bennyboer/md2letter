//! Transform the parsed blocks into a Letter document model tree.

use crate::parser::{
    CodeBlock, FunctionBlock, HeadingBlock, ImageBlock, ListBlock, ListNodeId, ListNodeKind,
    ListNodeStyle, ListTree, ParsedBlock, ParsedBlockKind, QuoteBlock, QuoteNodeId, QuoteNodeKind,
    QuoteTree, TableBlock, TableCell, TableRow, TextBlock, TextNodeId, TextNodeKind, TextTree,
};
use crate::transformer::result::TransformResult;
use crate::transformer::tree::{LetterScriptNodeId, LetterScriptNodeKind, LetterScriptTree};
use crate::util::SourceSpan;

mod result;
mod tree;

pub(crate) fn transform(
    blocks: impl Iterator<Item = ParsedBlock>,
) -> TransformResult<LetterScriptTree> {
    let mut tree = LetterScriptTree::new();

    transform_blocks(&mut tree, blocks);

    Ok(tree)
}

fn transform_blocks(tree: &mut LetterScriptTree, blocks: impl Iterator<Item = ParsedBlock>) {
    let mut node_stack = vec![tree.root_id()];

    for block in blocks {
        transform_block(tree, &mut node_stack, block);
    }
}

fn transform_block(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    block: ParsedBlock,
) {
    let span = block.span().clone();

    match block.into_kind() {
        ParsedBlockKind::Text(text_block) => {
            transform_text_block(tree, node_stack, text_block, span)
        }
        ParsedBlockKind::List(list_block) => {
            transform_list_block(tree, node_stack, list_block, span)
        }
        ParsedBlockKind::Heading(heading_block) => {
            transform_heading_block(tree, node_stack, heading_block, span)
        }
        ParsedBlockKind::Table(table_block) => {
            transform_table_block(tree, node_stack, table_block, span)
        }
        ParsedBlockKind::Image(image_block) => {
            transform_image_block(tree, node_stack, image_block, span)
        }
        ParsedBlockKind::Quote(quote_block) => {
            transform_quote_block(tree, node_stack, quote_block, span)
        }
        ParsedBlockKind::Code(code_block) => {
            transform_code_block(tree, node_stack, code_block, span)
        }
        ParsedBlockKind::Function(function_block) => {
            transform_function_block(tree, node_stack, function_block, span)
        }
        ParsedBlockKind::HorizontalRule => transform_horizontal_rule(tree, node_stack, span),
    }
}

fn transform_list_block(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    block: ListBlock,
    span: SourceSpan,
) {
    let list_tree = block.into_tree();
    let root = list_tree.root();
    transform_list_item(tree, node_stack, &list_tree, root.id(), span.clone());
}

fn transform_list_item(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    list_tree: &ListTree,
    item_node_id: ListNodeId,
    span: SourceSpan,
) {
    let list_node = list_tree.get_node(item_node_id);

    match list_node.kind() {
        ListNodeKind::Parent => {
            let is_ordered = if let ListNodeStyle::Ordered = list_node.style() {
                true
            } else {
                false
            };

            let list_node_id = tree.register_node(
                *node_stack.last().unwrap(),
                LetterScriptNodeKind::List {
                    ordered: is_ordered,
                },
                span.clone(),
            );

            node_stack.push(list_node_id);
            {
                for child_id in list_node.children() {
                    transform_list_item(tree, node_stack, list_tree, *child_id, span.clone());
                }
            }
            node_stack.pop();
        }
        ListNodeKind::Leaf { text_tree } => {
            let list_item_node_id = tree.register_node(
                *node_stack.last().unwrap(),
                LetterScriptNodeKind::ListItem,
                span.clone(),
            );

            node_stack.push(list_item_node_id);
            {
                transform_text_tree(tree, node_stack, text_tree);
            }
            node_stack.pop();
        }
    }
}

fn transform_heading_block(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    block: HeadingBlock,
    span: SourceSpan,
) {
    let mut current_level = 1;
    for node_id in node_stack.iter() {
        if let LetterScriptNodeKind::Section = tree.get_node(*node_id).kind() {
            current_level += 1;
        }
    }

    if block.level() > current_level {
        for _ in 0..(block.level() - current_level) {
            let section_node_id = tree.register_node(
                *node_stack.last().unwrap(),
                LetterScriptNodeKind::Section,
                span.clone(),
            );

            node_stack.push(section_node_id);
        }
    } else if block.level() < current_level {
        for _ in 0..(current_level - block.level()) {
            node_stack.pop();
        }
    }

    let heading_node_id = tree.register_node(
        *node_stack.last().unwrap(),
        LetterScriptNodeKind::Heading,
        span,
    );

    node_stack.push(heading_node_id);
    {
        transform_text_tree(tree, node_stack, block.text_tree());
    }
    node_stack.pop();
}

fn transform_table_block(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    block: TableBlock,
    span: SourceSpan,
) {
    let table_node_id = tree.register_node(
        *node_stack.last().unwrap(),
        LetterScriptNodeKind::Table,
        span.clone(),
    );

    node_stack.push(table_node_id);
    {
        transform_table_header_row(tree, node_stack, block.header_row(), span.clone());

        for row_index in 0..block.row_count() {
            let row = block.get_row(row_index).unwrap();
            transform_table_row(tree, node_stack, row, span.clone());
        }
    }
    node_stack.pop();
}

fn transform_table_row(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    row: &TableRow,
    span: SourceSpan,
) {
    let table_row_node_id = tree.register_node(
        *node_stack.last().unwrap(),
        LetterScriptNodeKind::TableRow,
        span.clone(),
    );

    node_stack.push(table_row_node_id);
    {
        for cell in row {
            transform_table_cell(tree, node_stack, cell, span.clone());
        }
    }
    node_stack.pop();
}

fn transform_table_header_row(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    header_row: &TableRow,
    span: SourceSpan,
) {
    let table_header_node_id = tree.register_node(
        *node_stack.last().unwrap(),
        LetterScriptNodeKind::TableHeaderRow,
        span.clone(),
    );

    node_stack.push(table_header_node_id);
    {
        for header_cell in header_row {
            transform_table_cell(tree, node_stack, header_cell, span.clone());
        }
    }
    node_stack.pop();
}

fn transform_table_cell(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    cell: &TableCell,
    span: SourceSpan,
) {
    let table_cell_node_id = tree.register_node(
        *node_stack.last().unwrap(),
        LetterScriptNodeKind::TableCell,
        span,
    );

    node_stack.push(table_cell_node_id);
    {
        let text_tree = cell.text_tree();
        transform_text_tree(tree, node_stack, text_tree);
    }
    node_stack.pop();
}

fn transform_image_block(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    block: ImageBlock,
    span: SourceSpan,
) {
    let node_id = tree.register_node(
        *node_stack.last().unwrap(),
        LetterScriptNodeKind::Image {
            src: block.src().to_string(),
        },
        span,
    );

    node_stack.push(node_id);

    let text_tree = block.text_tree();
    transform_text_tree(tree, node_stack, text_tree);

    node_stack.pop();
}

fn transform_quote_block(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    block: QuoteBlock,
    span: SourceSpan,
) {
    let quote_tree = block.into_tree();
    transform_quote_node(tree, node_stack, &quote_tree, quote_tree.root().id(), &span);
}

fn transform_quote_node(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    quote_tree: &QuoteTree,
    quote_node_id: QuoteNodeId,
    span: &SourceSpan,
) {
    let quote_node = quote_tree.get_node(quote_node_id);

    match quote_node.kind() {
        QuoteNodeKind::Parent => {
            let parent_id = *node_stack.last().unwrap();
            let node_id = tree.register_node(parent_id, LetterScriptNodeKind::Quote, span.clone());

            node_stack.push(node_id);
            for child_id in quote_node.children() {
                transform_quote_node(tree, node_stack, quote_tree, *child_id, span);
            }
            node_stack.pop();
        }
        QuoteNodeKind::Leaf { text_tree } => {
            transform_text_tree(tree, node_stack, text_tree);
        }
    }
}

fn transform_code_block(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    block: CodeBlock,
    span: SourceSpan,
) {
    let language = block.language().as_ref().map(|s| s.to_string());
    let src = block.src().to_string();

    let parent_id = *node_stack.last().unwrap();
    let node_id = tree.register_node(
        parent_id,
        LetterScriptNodeKind::Code { language },
        span.clone(),
    );
    tree.register_node(node_id, LetterScriptNodeKind::Text(src), span);
}

fn transform_function_block(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    block: FunctionBlock,
    span: SourceSpan,
) {
    let name = block.name().to_string();
    let parameters = block.parameters().clone();

    let parent_id = *node_stack.last().unwrap();
    tree.register_node(
        parent_id,
        LetterScriptNodeKind::Function { name, parameters },
        span,
    );
}

fn transform_horizontal_rule(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    span: SourceSpan,
) {
    let parent_id = *node_stack.last().unwrap();
    tree.register_node(parent_id, LetterScriptNodeKind::HorizontalRule, span);
}

fn transform_text_block(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    block: TextBlock,
    span: SourceSpan,
) {
    let paragraph_node_id = tree.register_node(
        *node_stack.last().unwrap(),
        LetterScriptNodeKind::Paragraph,
        span,
    );

    node_stack.push(paragraph_node_id);
    {
        let text_tree = block.into_tree();
        transform_text_tree(tree, node_stack, &text_tree);
    }
    node_stack.pop();
}

fn transform_text_tree(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    text_tree: &TextTree,
) {
    let root = text_tree.root();
    for child_id in root.children() {
        transform_text_node(tree, node_stack, &text_tree, *child_id);
    }
}

fn transform_text_node(
    tree: &mut LetterScriptTree,
    node_stack: &mut Vec<LetterScriptNodeId>,
    text_tree: &TextTree,
    text_node_id: TextNodeId,
) {
    let parent_id = *node_stack.last().unwrap();
    let text_node = text_tree.get_node(text_node_id);
    let span = text_node.span().clone();

    let node_kind = match text_node.kind() {
        TextNodeKind::Text { src } => LetterScriptNodeKind::Text(src.clone()),
        TextNodeKind::Bold => LetterScriptNodeKind::Bold,
        TextNodeKind::Italic => LetterScriptNodeKind::Italic,
        TextNodeKind::Code => LetterScriptNodeKind::Code { language: None },
        TextNodeKind::Link { target } => LetterScriptNodeKind::Link {
            target: target.clone(),
        },
        TextNodeKind::Image { src } => LetterScriptNodeKind::Image { src: src.clone() },
        TextNodeKind::Function { name, parameters } => LetterScriptNodeKind::Function {
            name: name.clone(),
            parameters: parameters.clone(),
        },
        _ => unreachable!(),
    };

    let node_id = tree.register_node(parent_id, node_kind, span);
    node_stack.push(node_id);

    for child_id in text_node.children() {
        transform_text_node(tree, node_stack, text_tree, *child_id);
    }

    node_stack.pop();
}

#[cfg(test)]
mod tests {
    use crate::categorizer::BlockCategorizer;
    use crate::parser::BlockParser;
    use crate::splitter::BlockSplitter;

    use super::*;

    fn to_letter_script_str(src: &'static str) -> String {
        let splitter = BlockSplitter::new(Box::new(src.as_bytes()));
        let categorizer = BlockCategorizer::new();
        let parser = BlockParser::new();

        let parsed_block_iterator = splitter
            .into_iter()
            .map(|block| categorizer.categorize(block))
            .map(|categorized_block| parser.parse(categorized_block).unwrap());

        let letter_script_tree = transform(parsed_block_iterator).unwrap();
        letter_script_tree.to_string()
    }

    #[test]
    fn should_transform_trivial_heading() {
        assert_eq!(
            to_letter_script_str("# This is a heading"),
            "\
<heading>
    This is a heading
</heading>
"
        );
    }

    #[test]
    fn should_transform_nested_headings() {
        assert_eq!(
            to_letter_script_str(
                "\
# This is a heading

## This is a subheading

With some content.

### This is a subsubheading

Here is some content.
"
            ),
            "\
<heading>
    This is a heading
</heading>
<section>
    <heading>
        This is a subheading
    </heading>
    <paragraph>
        With some content.
    </paragraph>
    <section>
        <heading>
            This is a subsubheading
        </heading>
        <paragraph>
            Here is some content. 
        </paragraph>
    </section>
</section>
"
        );
    }

    #[test]
    fn should_transform_text() {
        assert_eq!(
            to_letter_script_str(
                "\
Hello World, this is **bold text**.
We can also format in *italic* or even both ***bold and italic***.
"
            ),
            "\
<paragraph>
    Hello World, this is 
    <b>
        bold text
    </b>
    . We can also format in 
    <i>
        italic
    </i>
     or even both 
    <i>
        <b>
            bold and italic
        </b>
    </i>
    . 
</paragraph>
"
        );
    }

    #[test]
    fn should_transform_list() {
        assert_eq!(
            to_letter_script_str(
                "\
- A simple list item
- And another one
    - Now we are nested - yeehaw!
- And a third one
    - and
        - nesting
            1. even
            2. further
"
            ),
            "\
<list>
    <list-item>
        A simple list item
    </list-item>
    <list-item>
        And another one
    </list-item>
    <list>
        <list-item>
            Now we are nested - yeehaw!
        </list-item>
    </list>
    <list-item>
        And a third one
    </list-item>
    <list>
        <list-item>
            and
        </list-item>
        <list>
            <list-item>
                nesting
            </list-item>
            <list ordered=\"true\">
                <list-item>
                    even
                </list-item>
                <list-item>
                    further
                </list-item>
            </list>
        </list>
    </list>
</list>
"
        );
    }

    #[test]
    fn should_transform_horizontal_rule() {
        assert_eq!(
            to_letter_script_str(
                "\
This is a paragraph.

---

This is another paragraph.
"
            ),
            "\
<paragraph>
    This is a paragraph.
</paragraph>
<horizontal-rule/>
<paragraph>
    This is another paragraph. 
</paragraph>
"
        );
    }

    #[test]
    fn should_transform_code_block() {
        assert_eq!(
            to_letter_script_str(
                "\
This is a paragraph.

```
This is a code block.

console.log('Hello World!');
```

This is another paragraph.
"
            ),
            "\
<paragraph>
    This is a paragraph.
</paragraph>
<code>
    This is a code block.
    
    console.log('Hello World!');
</code>
<paragraph>
    This is another paragraph. 
</paragraph>
"
        );
    }

    #[test]
    fn should_transform_quote() {
        assert_eq!(
            to_letter_script_str(
                "\
This is a paragraph.

> This is a quote.
> It can span multiple lines.
>> And it can be nested.
"
            ),
            "\
<paragraph>
    This is a paragraph.
</paragraph>
<quote>
    This is a quote. It can span multiple lines.
    <quote>
        And it can be nested.
    </quote>
</quote>
"
        );
    }

    #[test]
    fn should_transform_table() {
        assert_eq!(
            to_letter_script_str(
                "\
| Column 1 | Column 2 |
| -------- | -------- |
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |
"
            ),
            "\
<table>
    <table-header-row>
        <table-cell>
            Column 1
        </table-cell>
        <table-cell>
            Column 2
        </table-cell>
    </table-header-row>
    <table-row>
        <table-cell>
            Cell 1
        </table-cell>
        <table-cell>
            Cell 2
        </table-cell>
    </table-row>
    <table-row>
        <table-cell>
            Cell 3
        </table-cell>
        <table-cell>
            Cell 4
        </table-cell>
    </table-row>
</table>
"
        );
    }

    #[test]
    fn should_transform_image() {
        assert_eq!(
            to_letter_script_str(
                "\
This is a paragraph.

![This is an image](image.png)
"
            ),
            "\
<paragraph>
    This is a paragraph.
</paragraph>
<image src=\"image.png\">
    This is an image
</image>
"
        );
    }

    #[test]
    fn should_transform_link() {
        assert_eq!(
            to_letter_script_str(
                "\
This is a [link](https://example.com).
"
            ),
            "\
<paragraph>
    This is a 
    <link target=\"https://example.com\">
        link
    </link>
    . 
</paragraph>
"
        );
    }

    #[test]
    fn should_transform_function() {
        assert_eq!(
            to_letter_script_str(
                "\
#image(
    width: 100px,
    height: 200px,
    src: image.png
)
"
            ),
            "\
<image height=\"200px\" src=\"image.png\" width=\"100px\">
</image>
"
        );
    }
}
