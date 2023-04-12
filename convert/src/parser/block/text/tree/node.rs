use std::fmt::{format, Display, Formatter};

use crate::parser::block::function::{FunctionName, FunctionParameters};
use crate::util::SourceSpan;

pub(crate) type NodeId = usize;

#[derive(Debug, PartialEq)]
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
        name: FunctionName,
        parameters: FunctionParameters,
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

impl Display for NodeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeKind::Root => write!(f, "[Root]"),
            NodeKind::Text { src } => write!(f, "[Text]({})", src),
            NodeKind::Bold => write!(f, "[Bold]"),
            NodeKind::Italic => write!(f, "[Italic]"),
            NodeKind::Code => write!(f, "[Code]"),
            NodeKind::Link { target } => write!(f, "[Link]({})", target),
            NodeKind::Image { src } => write!(f, "[Image]({})", src),
            NodeKind::Function { name, parameters } => {
                let mut param_strings = parameters
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key, value))
                    .collect::<Vec<_>>();
                param_strings.sort();

                write!(f, "[Function]({}, {})", name, param_strings.join(", "))
            }
        }
    }
}
