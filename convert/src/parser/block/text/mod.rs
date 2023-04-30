pub(crate) use self::tree::{TextNodeId, TextNodeKind, TextTree};

pub(crate) mod tree;

#[derive(Debug)]
pub(crate) struct TextBlock {
    tree: TextTree,
}

impl TextBlock {
    pub(crate) fn new(tree: TextTree) -> Self {
        Self { tree }
    }

    pub(crate) fn tree(&self) -> &TextTree {
        &self.tree
    }

    pub(crate) fn into_tree(self) -> TextTree {
        self.tree
    }
}
