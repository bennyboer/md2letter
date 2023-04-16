use crate::parser::block::text;

#[derive(Debug)]
pub(crate) struct TableCell {
    text_tree: text::Tree,
}

impl TableCell {
    pub fn new(text_tree: text::Tree) -> Self {
        Self { text_tree }
    }

    pub fn text_tree(&self) -> &text::Tree {
        &self.text_tree
    }
}
