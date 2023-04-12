pub(crate) use self::tree::{Node, NodeId, NodeKind, Tree};

pub(crate) mod tree;

#[derive(Debug)]
pub(crate) struct TextBlock {
    tree: Tree,
}

impl TextBlock {
    pub(crate) fn new(tree: Tree) -> Self {
        Self { tree }
    }

    pub(crate) fn tree(&self) -> &Tree {
        &self.tree
    }
    
    pub(crate) fn into_tree(self) -> Tree {
        self.tree
    }
}
