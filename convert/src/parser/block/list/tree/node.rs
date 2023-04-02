use crate::parser::block::text;

pub(crate) type ListNodeId = usize;

#[derive(Debug)]
pub(crate) enum ListNodeKind {
    Parent,
    Leaf { text_tree: text::Tree },
}

#[derive(Debug)]
pub(crate) enum ListNodeStyle {
    Ordered,
    Unordered,
}

#[derive(Debug)]
pub(crate) struct ListNode {
    id: ListNodeId,
    kind: ListNodeKind,
    children: Vec<ListNodeId>,
    style: ListNodeStyle,
}
