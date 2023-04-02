use super::text;

#[derive(Debug)]
pub(crate) struct HeadingBlock {
    level: usize,
    text_tree: text::Tree,
}
