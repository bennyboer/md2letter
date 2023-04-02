extern crate core;

use std::error::Error;
use std::io::Read;

use parser::BlockParser;

use crate::{categorizer::BlockCategorizer, splitter::BlockSplitter};

mod categorizer;
mod parser;
mod render;
mod source_position;
mod source_span;
mod splitter;
mod tokenizer;
mod transformer;

pub type ConvertResult<T> = Result<T, Box<dyn Error>>;

pub fn convert(reader: Box<dyn Read>) -> ConvertResult<String> {
    let splitter = BlockSplitter::new(reader);
    let categorizer = BlockCategorizer::new();
    let parser = BlockParser::new();

    splitter
        .into_iter()
        .map(|block| categorizer.categorize(block))
        .map(|categorized_block| parser.parse(categorized_block))
        .for_each(|parsed_block| println!("Parsed block '{:?}'", parsed_block));

    Ok("".to_string())
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn test() {
        let src = "
        # This is a heading

        This is a paragraph.
        With some **bold** and *italic* formatting.
        ";

        let _result = convert(Box::new(BufReader::new(src.as_bytes())));
    }
}
