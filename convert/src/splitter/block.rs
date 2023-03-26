use crate::source_span::SourceSpan;

#[derive(Debug)]
pub(crate) struct SplitterBlock {
    src: String,
    span: SourceSpan,
}

impl SplitterBlock {
    pub fn new(src: String, span: SourceSpan) -> Self {
        Self { src, span }
    }

    pub(crate) fn src(&self) -> &str {
        &self.src
    }

    pub(crate) fn span(&self) -> &SourceSpan {
        &self.span
    }
}
