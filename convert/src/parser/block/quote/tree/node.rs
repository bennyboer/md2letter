use crate::parser::block::text;

pub(crate) type QuoteNodeId = usize;

#[derive(Debug)]
pub(crate) enum QuoteNodeKind {
    Parent,
    Leaf { text_tree: text::TextTree },
}

#[derive(Debug)]
pub(crate) struct QuoteNode {
    id: QuoteNodeId,
    kind: QuoteNodeKind,
    children: Vec<QuoteNodeId>,
}

impl QuoteNode {
    pub fn new(id: QuoteNodeId, kind: QuoteNodeKind) -> Self {
        Self {
            id,
            kind,
            children: Vec::new(),
        }
    }

    pub fn id(&self) -> QuoteNodeId {
        self.id
    }

    pub fn kind(&self) -> &QuoteNodeKind {
        &self.kind
    }

    pub fn children(&self) -> &[QuoteNodeId] {
        &self.children
    }

    pub fn register_child(&mut self, child_id: QuoteNodeId) {
        self.children.push(child_id);
    }
}

impl QuoteNodeKind {
    pub(crate) fn to_string(&self, level: usize) -> String {
        match self {
            QuoteNodeKind::Parent => "[Parent]".to_string(),
            QuoteNodeKind::Leaf { text_tree } => {
                let text_tree_str = text_tree.to_string();
                let text_tree_lines: Vec<&str> = text_tree_str.lines().collect();
                let mut text_tree_representation = String::new();
                for line in text_tree_lines.into_iter().skip(1) {
                    let indent = "  ".repeat(level);
                    text_tree_representation.push_str(&format!("{}{}\n", indent, line));
                }

                format!("[Leaf]\n{}", text_tree_representation.trim_end()).to_string()
            }
        }
    }
}
