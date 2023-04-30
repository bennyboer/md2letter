use crate::parser::block::text;

#[derive(Debug)]
pub(crate) struct TableCell {
    text_tree: text::TextTree,
}

impl TableCell {
    pub fn new(text_tree: text::TextTree) -> Self {
        Self { text_tree }
    }

    pub fn text_tree(&self) -> &text::TextTree {
        &self.text_tree
    }
}
