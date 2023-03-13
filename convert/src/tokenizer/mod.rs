use std::io::{BufReader, Read};

use utf8_chars::BufReadCharsExt;

use crate::source_position::SourcePosition;
use crate::tokenizer::result::TokenizeResult;
use crate::tokenizer::state::{StartState, State, StateStackOperation};
use crate::tokenizer::token::Token;

mod result;
mod state;
mod token;

pub(crate) struct Tokenizer {
    reader: BufReader<Box<dyn Read>>,
    look_ahead_buffer: Vec<char>,
    current_position: SourcePosition,
}

impl Tokenizer {
    pub(crate) fn new(reader: Box<dyn Read>) -> Self {
        Self {
            reader: BufReader::new(Box::new(reader)),
            look_ahead_buffer: Vec::new(),
            current_position: SourcePosition::zero(),
        }
    }

    pub(crate) fn source_position(&self) -> SourcePosition {
        self.current_position.clone()
    }

    fn read_next(&mut self) -> Option<char> {
        let next_char = if let Some(c) = self.look_ahead_buffer.pop() {
            return Some(c);
        } else {
            self.reader.read_char().ok().flatten()
        };

        if let Some(c) = next_char {
            if c == '\r' {
                return self.read_next();
            }

            if c == '\n' {
                self.current_position.line += 1;
                self.current_position.column = 1;
            } else {
                self.current_position.column += 1;
            }
        }

        next_char
    }

    fn _look_ahead(&mut self, count: usize) -> Option<&[char]> {
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
            let current_state = state_stack.last_mut().unwrap();

            let mut end_loop = false;
            let result = match self.read_next() {
                Some(c) => current_state.process(c, self),
                None => {
                    end_loop = true;
                    current_state.end_of_input(self)
                }
            };

            update_state_stack(&mut state_stack, result.state_stack_operation);

            if let Some(err) = result.err {
                return Some(Err(err));
            }

            if let Some(token) = result.token {
                return Some(Ok(token));
            }

            if end_loop {
                return None;
            }
        }
    }
}

fn update_state_stack(state_stack: &mut Vec<Box<dyn State>>, operation: StateStackOperation) {
    match operation {
        StateStackOperation::Push(state) => {
            state_stack.push(state);
        }
        StateStackOperation::Pop => {
            state_stack.pop();
        }
        StateStackOperation::Replace(state) => {
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

    use crate::source_span::SourceSpan;
    use crate::tokenizer::token::TokenKind::{
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

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, HeadingLevel(0));
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
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
            tokenizer.next().unwrap().unwrap().kind,
            Text("Paragraph".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BlockSeparator);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
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

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEmphasis);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("This is emphasized".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEmphasis);
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

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEmphasis);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("This is emphasized".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEmphasis);
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

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, HeadingLevel(0));
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" This is a ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEmphasis);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("heading".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEmphasis);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" with emphasis".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BlockSeparator);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("This is a paragraph with ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEmphasis);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("bold".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEmphasis);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" emphasis.".to_string())
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("It should work.".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BlockSeparator);
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, HeadingLevel(1));
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" This is a subheading".to_string())
        );
        assert!(tokenizer.next().is_none());

        Ok(())
    }

    #[test]
    fn should_include_source_span_in_tokens() -> Result<(), Box<dyn Error>> {
        let src = "## Heading test

Paragraph with **bold** emphasis.";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Token::new(
                HeadingLevel(1),
                SourceSpan {
                    start: SourcePosition { line: 1, column: 1 },
                    end: SourcePosition { line: 1, column: 3 }
                }
            )
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Token::new(
                Text(" Heading test".to_string()),
                SourceSpan {
                    start: SourcePosition { line: 1, column: 3 },
                    end: SourcePosition {
                        line: 1,
                        column: 16
                    }
                }
            )
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Token::new(
                BlockSeparator,
                SourceSpan {
                    start: SourcePosition { line: 2, column: 1 },
                    end: SourcePosition { line: 3, column: 1 }
                }
            )
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Token::new(
                Text("Paragraph with ".to_string()),
                SourceSpan {
                    start: SourcePosition { line: 3, column: 1 },
                    end: SourcePosition {
                        line: 3,
                        column: 16
                    }
                }
            )
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Token::new(
                BoldEmphasis,
                SourceSpan {
                    start: SourcePosition {
                        line: 3,
                        column: 16
                    },
                    end: SourcePosition {
                        line: 3,
                        column: 18
                    }
                }
            )
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Token::new(
                Text("bold".to_string()),
                SourceSpan {
                    start: SourcePosition {
                        line: 3,
                        column: 18
                    },
                    end: SourcePosition {
                        line: 3,
                        column: 22
                    }
                }
            )
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Token::new(
                BoldEmphasis,
                SourceSpan {
                    start: SourcePosition {
                        line: 3,
                        column: 22
                    },
                    end: SourcePosition {
                        line: 3,
                        column: 24
                    }
                }
            )
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap(),
            Token::new(
                Text(" emphasis.".to_string()),
                SourceSpan {
                    start: SourcePosition {
                        line: 3,
                        column: 24
                    },
                    end: SourcePosition {
                        line: 3,
                        column: 34
                    }
                }
            )
        );
        assert!(tokenizer.next().is_none());

        Ok(())
    }
}
