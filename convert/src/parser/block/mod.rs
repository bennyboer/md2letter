use crate::util::SourceSpan;

pub(crate) use self::{
    code::CodeBlock, function::FunctionBlock, heading::HeadingBlock, image::ImageBlock,
    list::ListBlock, quote::QuoteBlock, table::TableBlock, text::TextBlock,
};

pub(crate) mod code;
pub(crate) mod function;
pub(crate) mod heading;
pub(crate) mod image;
pub(crate) mod list;
pub(crate) mod quote;
pub(crate) mod table;
pub(crate) mod text;

#[derive(Debug)]
pub(crate) struct ParsedBlock {
    kind: ParsedBlockKind,
    span: SourceSpan,
}

#[derive(Debug)]
pub(crate) enum ParsedBlockKind {
    Text(TextBlock),
    List(ListBlock),
    Heading(HeadingBlock),
    Table(TableBlock),
    Image(ImageBlock),
    Quote(QuoteBlock),
    Code(CodeBlock),
    Function(FunctionBlock),
    HorizontalRule,
}

impl ParsedBlock {
    pub(crate) fn new(kind: ParsedBlockKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }

    pub(crate) fn kind(&self) -> &ParsedBlockKind {
        &self.kind
    }

    pub(crate) fn into_kind(self) -> ParsedBlockKind {
        self.kind
    }

    pub(crate) fn span(&self) -> &SourceSpan {
        &self.span
    }

    pub(crate) fn is_text(&self) -> bool {
        matches!(self.kind, ParsedBlockKind::Text(_))
    }

    pub(crate) fn is_list(&self) -> bool {
        matches!(self.kind, ParsedBlockKind::List(_))
    }

    pub(crate) fn is_heading(&self) -> bool {
        matches!(self.kind, ParsedBlockKind::Heading(_))
    }

    pub(crate) fn is_table(&self) -> bool {
        matches!(self.kind, ParsedBlockKind::Table(_))
    }

    pub(crate) fn is_image(&self) -> bool {
        matches!(self.kind, ParsedBlockKind::Image(_))
    }

    pub(crate) fn is_quote(&self) -> bool {
        matches!(self.kind, ParsedBlockKind::Quote(_))
    }

    pub(crate) fn is_code(&self) -> bool {
        matches!(self.kind, ParsedBlockKind::Code(_))
    }

    pub(crate) fn is_function(&self) -> bool {
        matches!(self.kind, ParsedBlockKind::Function(_))
    }

    pub(crate) fn is_horizontal_rule(&self) -> bool {
        matches!(self.kind, ParsedBlockKind::HorizontalRule)
    }
}
