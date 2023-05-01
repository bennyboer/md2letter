extern crate core;

use std::error::Error;
use std::io::Read;

use parser::BlockParser;

use crate::parser::{ParseError, ParsedBlock};
use crate::transformer::transform;
use crate::{categorizer::BlockCategorizer, splitter::BlockSplitter};

mod categorizer;
mod parser;
mod render;
mod splitter;
mod transformer;
pub(crate) mod util;

pub type ConvertResult<T> = Result<T, Box<dyn Error>>;

pub fn convert(reader: Box<dyn Read>) -> ConvertResult<String> {
    let splitter = BlockSplitter::new(reader);
    let categorizer = BlockCategorizer::new();
    let parser = BlockParser::new();

    let blocks_result: Result<Vec<ParsedBlock>, ParseError> = splitter
        .into_iter()
        .map(|block| categorizer.categorize(block))
        .map(|categorized_block| parser.parse(categorized_block))
        .collect();

    let blocks = if let Ok(blocks) = blocks_result {
        blocks
    } else {
        return Err(format!("Failed to parse block: {:?}", blocks_result).into());
    };

    let tree = transform(blocks.into_iter()).map_err(|e| e.message)?;

    // TODO Render tree properly using an XML/HTML formatter

    Ok(tree.to_string())
}
