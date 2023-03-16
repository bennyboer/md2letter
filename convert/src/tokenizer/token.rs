use crate::source_span::SourceSpan;

#[derive(Debug, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) source_span: SourceSpan,
}

#[derive(Debug, PartialEq)]
pub(crate) enum TokenKind {
    BlockSeparator,
    HeadingLevel(usize),
    Text(String),
    Emphasis(EmphasisType),
    HorizontalRule,
}

#[derive(Debug, PartialEq)]
pub(crate) enum EmphasisType {
    BoldOrItalic,
    Code,
}

impl Token {
    pub(crate) fn new(kind: TokenKind, source_span: SourceSpan) -> Self {
        Self { kind, source_span }
    }
}
