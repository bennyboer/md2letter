use std::io::{BufReader, Read};

use utf8_chars::BufReadCharsExt;

use crate::tokenizer::result::TokenizeResult;
use crate::tokenizer::state::{StartState, State, StateStackOperation};
use crate::tokenizer::token::Token;

mod result;
mod state;
mod token;

pub(crate) struct Tokenizer {
    reader: BufReader<Box<dyn Read>>,
    look_ahead_buffer: Vec<char>,
}

impl Tokenizer {
    pub(crate) fn new(reader: Box<dyn Read>) -> Self {
        Self {
            reader: BufReader::new(Box::new(reader)),
            look_ahead_buffer: Vec::new(),
        }
    }

    fn read_next(&mut self) -> Option<char> {
        if let Some(c) = self.look_ahead_buffer.pop() {
            return Some(c);
        }

        self.reader.read_char().ok().flatten()
    }

    fn look_ahead(&mut self, count: usize) -> Option<&[char]> {
        while self.look_ahead_buffer.len() < count {
            if let Some(c) = self.read_next() {
                self.look_ahead_buffer.insert(0, c);
            } else {
                break;
            }
        }

        if self.look_ahead_buffer.len() >= count {
            Some(&self.look_ahead_buffer[..count])
        } else {
            None
        }
    }

    fn mark_char_as_unconsumed(&mut self, c: char) {
        self.look_ahead_buffer.push(c)
    }
}

impl Iterator for Tokenizer {
    type Item = TokenizeResult<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut state_stack: Vec<Box<dyn State>> = vec![Box::new(StartState::new())];

        loop {
            match self.read_next() {
                Some(c) => {
                    println!("\n------------------");
                    println!(
                        "State stack: {:?}",
                        state_stack.iter().map(|s| s.name()).collect::<Vec<&str>>()
                    );
                    let current_state = state_stack.last_mut().unwrap();

                    println!(
                        "Processing char: '{}' with state: {}",
                        c,
                        current_state.name()
                    );
                    let result = current_state.process(c, self);

                    update_state_stack(&mut state_stack, result.state_stack_operation);

                    if let Some(err) = result.err {
                        return Some(Err(err));
                    }

                    if let Some(token) = result.token {
                        println!("Found token: {:?}", token);
                        return Some(Ok(token));
                    }
                }
                None => return None,
            };
        }
    }
}

fn update_state_stack(state_stack: &mut Vec<Box<dyn State>>, operation: StateStackOperation) {
    match operation {
        StateStackOperation::Push(state) => {
            println!("Pushing state: {}", state.name());
            state_stack.push(state);
        }
        StateStackOperation::Pop => {
            println!("Popping state: {}", state_stack.last().unwrap().name());
            state_stack.pop();
        }
        StateStackOperation::Replace(state) => {
            println!(
                "Replacing state {} with {}",
                state_stack.last().unwrap().name(),
                state.name()
            );
            state_stack.pop();
            state_stack.push(state);
        }
        StateStackOperation::None => {}
    }
}

#[cfg(test)]
mod test {
    use std::error::Error;
    use std::io::BufReader;

    use crate::tokenizer::token::Token::{
        BlockSeparator, BoldEmphasis, HeadingLevel, ItalicEmphasis, Text,
    };

    use super::*;

    #[test]
    fn tokenize_heading() -> Result<(), Box<dyn Error>> {
        let src = "
        # This is a heading
        ";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap(), HeadingLevel(0));
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text(" This is a heading".to_string())
        );
        assert!(tokenizer.next().is_none());

        Ok(())
    }

    #[test]
    fn tokenize_block_separator() -> Result<(), Box<dyn Error>> {
        let src = "
        Paragraph

        Paragraph
        ";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text("Paragraph".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap(), BlockSeparator);
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text("Paragraph".to_string())
        );
        assert!(tokenizer.next().is_none());

        Ok(())
    }

    #[test]
    fn tokenize_italic_emphasis() -> Result<(), Box<dyn Error>> {
        let src = "
        *This is emphasized*
        ";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap(), ItalicEmphasis);
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text("This is emphasized".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap(), ItalicEmphasis);
        assert!(tokenizer.next().is_none());

        Ok(())
    }

    #[test]
    fn tokenize_bold_emphasis() -> Result<(), Box<dyn Error>> {
        let src = "
        **This is emphasized**
        ";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap(), BoldEmphasis);
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text("This is emphasized".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap(), BoldEmphasis);
        assert!(tokenizer.next().is_none());

        Ok(())
    }

    #[test]
    fn tokenize_complex() -> Result<(), Box<dyn Error>> {
        let src = "
        # This is a *heading* with emphasis

        This is a paragraph with **bold** emphasis.
        It should work.

        ## This is a subheading


        ";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap(), HeadingLevel(0));
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text(" This is a ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap(), ItalicEmphasis);
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text("heading".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap(), ItalicEmphasis);
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text(" with emphasis".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap(), BlockSeparator);
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text("This is a paragraph with ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap(), BoldEmphasis);
        assert_eq!(tokenizer.next().unwrap().unwrap(), Text("bold".to_string()));
        assert_eq!(tokenizer.next().unwrap().unwrap(), BoldEmphasis);
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text(" emphasis.".to_string())
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text("It should work.".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap(), BlockSeparator);
        assert_eq!(tokenizer.next().unwrap().unwrap(), HeadingLevel(1));
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Text(" This is a subheading".to_string())
        );
        assert!(tokenizer.next().is_none());

        Ok(())
    }

    #[test]
    fn should_include_source_span_in_tokens() -> Result<(), Box<dyn Error>> {
        // TODO Test that tokens include the exact start end end position in the source.
        Err("Not implemented".into())
    }
}
