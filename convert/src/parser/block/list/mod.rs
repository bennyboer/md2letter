pub(crate) use self::tree::{ListNodeKind, ListNodeStyle, ListTree, ListNodeId};

mod tree;

#[derive(Debug)]
pub(crate) struct ListBlock {
    tree: ListTree,
}

impl ListBlock {
    pub fn new(tree: ListTree) -> Self {
        Self { tree }
    }

    pub fn into_tree(self) -> ListTree {
        self.tree
    }
}
