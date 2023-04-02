use self::tree::QuoteTree;

mod tree;

#[derive(Debug)]
pub(crate) struct QuoteBlock {
    tree: QuoteTree,
}
