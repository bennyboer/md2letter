use crate::parser::block::text;

#[derive(Debug)]
pub(crate) struct TableCell {
    text_tree: text::Tree,
}
