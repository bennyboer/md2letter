use crate::parser::block::text;

pub(crate) type ListNodeId = usize;

pub(crate) enum ListNodeKind {
    Parent,
    Leaf { text_tree: text::Tree },
}

pub(crate) enum ListNodeStyle {
    Ordered,
    Unordered,
}

pub(crate) struct ListNode {
    id: ListNodeId,
    kind: ListNodeKind,
    children: Vec<ListNodeId>,
    style: ListNodeStyle,
}
