use super::text;

pub(crate) struct HeadingBlock {
    level: usize,
    text_tree: text::Tree,
}
