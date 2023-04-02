pub(crate) use self::tree::Tree;

pub(crate) mod tree;

#[derive(Debug)]
pub(crate) struct TextBlock {
    tree: Tree,
}
