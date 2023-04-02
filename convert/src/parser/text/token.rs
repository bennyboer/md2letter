use crate::parser::block::function::{FunctionName, FunctionParameters};
use crate::util::{SourcePosition, SourceSpan};

#[derive(Debug, PartialEq)]
pub(crate) struct Token {
    kind: TokenKind,
    span: SourceSpan,
}

#[derive(Debug, PartialEq)]
pub(crate) enum TokenKind {
    Error {
        message: String,
        source_position: SourcePosition,
    },
    Text(String),
    Link {
        label: String,
        target: String,
    },
    Image {
        label: String,
        src: String,
    },
    Function {
        name: FunctionName,
        parameters: FunctionParameters,
    },
    BoldStart,
    BoldEnd,
    ItalicStart,
    ItalicEnd,
    CodeStart,
    CodeEnd,
}

impl Token {
    pub(crate) fn new(kind: TokenKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    pub(crate) fn kind(&self) -> &TokenKind {
        &self.kind
    }

    pub(crate) fn span(&self) -> &SourceSpan {
        &self.span
    }
}
