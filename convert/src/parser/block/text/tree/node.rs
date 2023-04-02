use std::collections::HashMap;

use crate::source_span::SourceSpan;

pub(crate) type NodeId = usize;

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

pub(crate) struct Node {
    id: NodeId,
    kind: NodeKind,
    children: Vec<NodeId>,
    span: SourceSpan,
}