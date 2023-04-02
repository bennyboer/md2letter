pub(crate) use self::tree::Tree;

pub(crate) mod tree;

pub(crate) struct TextBlock {
    tree: Tree,
}
