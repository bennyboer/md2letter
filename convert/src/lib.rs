extern crate core;

use std::error::Error;
use std::io::Read;

use tokenizer::Tokenizer;

mod source_position;
mod source_span;
mod tokenizer;

pub type ConvertResult<T> = Result<T, Box<dyn Error>>;

pub fn convert(reader: Box<dyn Read>) -> ConvertResult<String> {
    let tokenizer = Tokenizer::new(reader);

    // TODO Replace below two lines by iterating over tokens instead.
    let tokens = tokenizer.map(|t| t.unwrap()).collect::<Vec<_>>();
    println!("{:#?}", tokens);

    // TODO Invoke parser to turn tokens into a flat list of blocks

    // TODO Invoke renderer to turn blocks into a tree (letter script format)

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
