use crate::source_span::SourceSpan;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BlockKind {
    Text,
    Heading,
    List,
    Table,
    Image,
    Quote,
    Code,
    Function,
    HorizontalRule,
}

#[derive(Debug)]
pub(crate) struct CategorizedBlock {
    kind: BlockKind,
    src: String,
    span: SourceSpan,
}

impl CategorizedBlock {
    pub(crate) fn new(kind: BlockKind, src: String, span: SourceSpan) -> Self {
        Self { kind, src, span }
    }

    pub(crate) fn kind(&self) -> &BlockKind {
        &self.kind
    }

    pub(crate) fn src(&self) -> &str {
        &self.src
    }

    pub(crate) fn span(&self) -> &SourceSpan {
        &self.span
    }
}
