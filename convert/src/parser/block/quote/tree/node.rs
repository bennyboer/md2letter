use crate::parser::block::text;

pub(crate) type QuoteNodeId = usize;

pub(crate) enum QuoteNodeKind {
    Parent,
    Leaf { text_tree: text::Tree },
}

pub(crate) struct QuoteNode {
    id: QuoteNodeId,
    kind: QuoteNodeKind,
    children: Vec<QuoteNodeId>,
}
