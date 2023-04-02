use self::tree::QuoteTree;

mod tree;

pub(crate) struct QuoteBlock {
    tree: QuoteTree,
}
