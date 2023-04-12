use std::fmt::{Display, Formatter};

use crate::parser::block::text;

pub(crate) type ListNodeId = usize;

#[derive(Debug)]
pub(crate) enum ListNodeKind {
    Parent,
    Leaf { text_tree: text::Tree },
}

#[derive(Debug, Copy, Clone)]
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

impl ListNode {
    pub fn new(id: ListNodeId, kind: ListNodeKind, style: ListNodeStyle) -> Self {
        Self {
            id,
            kind,
            children: Vec::new(),
            style,
        }
    }

    pub fn id(&self) -> ListNodeId {
        self.id
    }

    pub fn kind(&self) -> &ListNodeKind {
        &self.kind
    }

    pub fn children(&self) -> &Vec<ListNodeId> {
        &self.children
    }

    pub fn style(&self) -> &ListNodeStyle {
        &self.style
    }

    pub(crate) fn register_child(&mut self, child: ListNodeId) {
        self.children.push(child);
    }
}

impl ListNodeKind {
    pub(crate) fn to_string(&self, level: usize) -> String {
        match self {
            ListNodeKind::Parent => "[Parent]".to_string(),
            ListNodeKind::Leaf { text_tree } => {
                let text_tree_str = text_tree.to_string();
                let text_tree_lines: Vec<&str> = text_tree_str.lines().collect();
                let mut text_tree_representation = String::new();
                for line in text_tree_lines.into_iter().skip(1) {
                    let indent = "  ".repeat(level);
                    text_tree_representation.push_str(&format!("{}{}\n", indent, line));
                }

                format!("[Item]\n{}", text_tree_representation.trim_end()).to_string()
            }
        }
    }
}

impl Display for ListNodeStyle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ListNodeStyle::Ordered => write!(f, "ordered"),
            ListNodeStyle::Unordered => write!(f, "unordered"),
        }
    }
}
