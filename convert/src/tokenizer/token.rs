#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    BlockSeparator,
    HeadingLevel(usize),
    Text(String),
    ItalicEmphasis,
    BoldEmphasis,
}
