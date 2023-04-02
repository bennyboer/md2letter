use crate::source_span::SourceSpan;

pub(crate) type LanguageIdentifier = String;

#[derive(Debug)]
pub(crate) struct CodeBlock {
    language: LanguageIdentifier,
    src: String,
    span: SourceSpan,
}
