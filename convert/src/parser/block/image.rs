use crate::parser::block::text;

pub(crate) type ImageSource = String;

#[derive(Debug)]
pub(crate) struct ImageBlock {
    text_tree: text::Tree,
    src: ImageSource,
}

impl ImageBlock {
    pub fn new(text_tree: text::Tree, src: ImageSource) -> Self {
        Self { text_tree, src }
    }

    pub fn text_tree(&self) -> &text::Tree {
        &self.text_tree
    }

    pub fn src(&self) -> &str {
        &self.src
    }
}
