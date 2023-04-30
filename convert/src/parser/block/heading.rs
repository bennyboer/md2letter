use super::text;

#[derive(Debug)]
pub(crate) struct HeadingBlock {
    level: usize,
    text_tree: text::TextTree,
}

impl HeadingBlock {
    pub fn new(level: usize, text_tree: text::TextTree) -> Self {
        Self { level, text_tree }
    }

    pub fn level(&self) -> usize {
        self.level
    }

    pub fn text_tree(&self) -> &text::TextTree {
        &self.text_tree
    }
}
