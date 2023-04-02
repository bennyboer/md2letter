use crate::source_span::SourceSpan;

use self::{
    code::CodeBlock, function::FunctionBlock, heading::HeadingBlock, image::ImageBlock,
    list::ListBlock, quote::QuoteBlock, table::TableBlock, text::TextBlock,
};

mod code;
mod function;
mod heading;
mod image;
mod list;
mod quote;
mod table;
mod text;

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
