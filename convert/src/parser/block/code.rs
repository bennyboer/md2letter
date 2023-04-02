use crate::source_span::SourceSpan;

pub(crate) type LanguageIdentifier = String;

pub(crate) struct CodeBlock {
    language: LanguageIdentifier,
    src: String,
    span: SourceSpan,
}
