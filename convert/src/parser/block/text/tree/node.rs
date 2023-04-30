use std::fmt::{Display, Formatter};

use crate::parser::block::function::{FunctionName, FunctionParameters};
use crate::util::SourceSpan;

pub(crate) type TextNodeId = usize;

#[derive(Debug, PartialEq)]
pub(crate) enum TextNodeKind {
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
pub(crate) struct TextNode {
    id: TextNodeId,
    kind: TextNodeKind,
    children: Vec<TextNodeId>,
    span: SourceSpan,
}

impl TextNode {
    pub(crate) fn new(id: TextNodeId, kind: TextNodeKind, span: SourceSpan) -> Self {
        Self {
            id,
            kind,
            children: Vec::new(),
            span,
        }
    }

    pub(crate) fn id(&self) -> TextNodeId {
        self.id
    }

    pub(crate) fn kind(&self) -> &TextNodeKind {
        &self.kind
    }

    pub(crate) fn children(&self) -> &[TextNodeId] {
        &self.children
    }

    pub(crate) fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn register_child(&mut self, child: TextNodeId) {
        self.children.push(child);
    }
}

impl Display for TextNodeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TextNodeKind::Root => write!(f, "[Root]"),
            TextNodeKind::Text { src } => write!(f, "[Text]({})", src),
            TextNodeKind::Bold => write!(f, "[Bold]"),
            TextNodeKind::Italic => write!(f, "[Italic]"),
            TextNodeKind::Code => write!(f, "[Code]"),
            TextNodeKind::Link { target } => write!(f, "[Link]({})", target),
            TextNodeKind::Image { src } => write!(f, "[Image]({})", src),
            TextNodeKind::Function { name, parameters } => {
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
