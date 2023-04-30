use std::collections::HashMap;

use crate::util::SourceSpan;

pub(crate) type LetterScriptNodeId = usize;

pub(crate) struct LetterScriptNode {
    id: LetterScriptNodeId,
    kind: LetterScriptNodeKind,
    children: Vec<LetterScriptNodeId>,
    span: SourceSpan,
}

pub(crate) enum LetterScriptNodeKind {
    Root,
    Text(String),
    Heading,
    Paragraph,
    Section,
    Image {
        src: String,
    },
    Quote,
    List {
        ordered: bool,
    },
    ListItem,
    HorizontalRule,
    Link {
        target: String,
    },
    Bold,
    Italic,
    Code {
        language: Option<String>,
    },
    Table,
    TableHeaderRow,
    TableRow,
    TableCell,
    Function {
        name: String,
        parameters: HashMap<String, String>,
    },
}

impl LetterScriptNode {
    pub(crate) fn new(
        id: LetterScriptNodeId,
        kind: LetterScriptNodeKind,
        span: SourceSpan,
    ) -> Self {
        Self {
            id,
            kind,
            children: Vec::new(),
            span,
        }
    }

    pub(crate) fn id(&self) -> LetterScriptNodeId {
        self.id
    }

    pub(crate) fn kind(&self) -> &LetterScriptNodeKind {
        &self.kind
    }

    pub(crate) fn children(&self) -> &[LetterScriptNodeId] {
        &self.children
    }

    pub(crate) fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn register_child(&mut self, child: LetterScriptNodeId) {
        self.children.push(child);
    }
}
