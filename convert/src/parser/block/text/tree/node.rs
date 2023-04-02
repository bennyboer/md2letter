use std::collections::HashMap;

use crate::util::SourceSpan;

pub(crate) type NodeId = usize;

#[derive(Debug)]
pub(crate) enum NodeKind {
    Root,
    Text {
        src: String,
    },
    Bold,
    Italic,
    Code,
    Link {
        target: String,
    },
    Image {
        src: String,
    },
    Function {
        name: String,
        parameters: HashMap<String, String>,
    },
}

#[derive(Debug)]
pub(crate) struct Node {
    id: NodeId,
    kind: NodeKind,
    children: Vec<NodeId>,
    span: SourceSpan,
}

impl Node {
    pub(crate) fn new(id: NodeId, kind: NodeKind, span: SourceSpan) -> Self {
        Self {
            id,
            kind,
            children: Vec::new(),
            span,
        }
    }

    pub(crate) fn id(&self) -> NodeId {
        self.id
    }

    pub(crate) fn kind(&self) -> &NodeKind {
        &self.kind
    }

    pub(crate) fn children(&self) -> &[NodeId] {
        &self.children
    }

    pub(crate) fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn register_child(&mut self, child: NodeId) {
        self.children.push(child);
    }
}
