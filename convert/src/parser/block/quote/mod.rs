pub(crate) use self::tree::{QuoteNodeId, QuoteNodeKind, QuoteTree};

mod tree;

#[derive(Debug)]
pub(crate) struct QuoteBlock {
    tree: QuoteTree,
}

impl QuoteBlock {
    pub fn new(tree: QuoteTree) -> Self {
        Self { tree }
    }

    pub fn into_tree(self) -> QuoteTree {
        self.tree
    }
}
