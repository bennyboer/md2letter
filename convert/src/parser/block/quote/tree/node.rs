use crate::parser::block::text;

pub(crate) type QuoteNodeId = usize;

#[derive(Debug)]
pub(crate) enum QuoteNodeKind {
    Parent,
    Leaf { text_tree: text::Tree },
}

#[derive(Debug)]
pub(crate) struct QuoteNode {
    id: QuoteNodeId,
    kind: QuoteNodeKind,
    children: Vec<QuoteNodeId>,
}
