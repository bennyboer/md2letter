use super::text;

#[derive(Debug)]
pub(crate) struct HeadingBlock {
    level: usize,
    text_tree: text::Tree,
}

impl HeadingBlock {
    pub fn new(level: usize, text_tree: text::Tree) -> Self {
        Self { level, text_tree }
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn text_tree(&self) -> &text::Tree {
        &self.text_tree
    }
}
