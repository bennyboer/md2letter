//! Split a string (or file) into blocks of text that are separated by one or more empty lines.

use std::collections::VecDeque;
use std::io::{BufReader, Read};

use utf8_chars::BufReadCharsExt;

pub(crate) use block::SplitterBlock;

use crate::util::SourcePosition;
use crate::util::SourceSpan;

mod block;

pub(crate) struct BlockSplitter {
    reader: BufReader<Box<dyn Read>>,
    unread_chars_buffer: VecDeque<char>,
    last_char_source_position: SourcePosition,
    next_char_source_position: SourcePosition,
}

impl BlockSplitter {
    pub(crate) fn new(reader: Box<dyn Read>) -> Self {
        Self {
            reader: BufReader::new(reader),
            unread_chars_buffer: VecDeque::new(),
            last_char_source_position: SourcePosition::zero(),
            next_char_source_position: SourcePosition::zero(),
        }
    }

    fn read_next_char(&mut self) -> Option<char> {
        let next_char = self.unread_chars_buffer.pop_front();

        let update_source_position = next_char.is_none();
        let next_char = next_char.or_else(|| self.reader.read_char().ok().flatten());

        if let Some(c) = next_char {
            if c == '\r' {
                return self.read_next_char();
            }

            if update_source_position {
                self.last_char_source_position = self.next_char_source_position.clone();

                if c == '\n' {
                    self.next_char_source_position.line += 1;
                    self.next_char_source_position.column = 1;
                } else {
                    self.next_char_source_position.column += 1;
                }
            }
        }

        next_char
    }

    pub fn last_char_source_position(&self) -> SourcePosition {
        self.last_char_source_position.clone()
    }

    pub fn next_char_source_position(&self) -> SourcePosition {
        self.next_char_source_position.clone()
    }

    fn push_unread_char(&mut self, c: char) {
        self.unread_chars_buffer.push_back(c);
    }
}

impl Iterator for BlockSplitter {
    type Item = SplitterBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let start_position = self.last_char_source_position();
        let mut end_position = self.next_char_source_position();
        let mut buffer = String::new();
        let mut newline_count = 0;
        let mut consecutive_backtick_counter = 0;
        let mut in_code_block = false;

        loop {
            let next_char = self.read_next_char();

            match next_char {
                None => {
                    return if buffer.is_empty() {
                        None
                    } else {
                        Some(SplitterBlock::new(
                            buffer,
                            SourceSpan::new(start_position, end_position),
                        ))
                    }
                }
                Some(c) => match c {
                    '\r' => {}
                    '\n' => {
                        newline_count += 1;
                        buffer.push(c);
                    }
                    ' ' | '\t' => {
                        // Do not reset newline count for whitespace chars
                        buffer.push(c);
                    }
                    _ => {
                        if newline_count >= 2 && !in_code_block {
                            self.push_unread_char(c);

                            let trimmed_string = buffer.trim().to_string();
                            return Some(SplitterBlock::new(
                                trimmed_string,
                                SourceSpan::new(start_position, end_position),
                            ));
                        }

                        if c == '`' {
                            if buffer.is_empty()
                                || consecutive_backtick_counter > 0
                                || in_code_block
                            {
                                consecutive_backtick_counter += 1;

                                if consecutive_backtick_counter == 3 {
                                    in_code_block = !in_code_block;
                                    consecutive_backtick_counter = 0;
                                }
                            }
                        } else {
                            consecutive_backtick_counter = 0;
                        }

                        newline_count = 0;
                        end_position = self.next_char_source_position();
                        buffer.push(c);
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_block() {
        let src = r#"This is a simple paragraph.
It contains of two lines and sentences."#;

        let mut splitter = BlockSplitter::new(Box::new(src.as_bytes()));

        let block = splitter.next().unwrap();
        assert_eq!(block.src(), src);
        assert_eq!(block.span().start.line, 1);
        assert_eq!(block.span().start.column, 1);
        assert_eq!(block.span().end.line, 2);
        assert_eq!(block.span().end.column, 40);

        assert!(splitter.next().is_none());
    }

    #[test]
    fn multiple_blocks() {
        let src = r#"This is a simple paragraph.
It contains of two lines and sentences.

This is a second paragraph.
It contains of three lines and sentences.
The paragraph is finalized by this sentence."#;

        let mut splitter = BlockSplitter::new(Box::new(src.as_bytes()));

        let first_block = splitter.next().unwrap();
        assert_eq!(
            first_block.src(),
            r#"This is a simple paragraph.
It contains of two lines and sentences."#
        );
        assert_eq!(first_block.span().start.line, 1);
        assert_eq!(first_block.span().start.column, 1);
        assert_eq!(first_block.span().end.line, 2);
        assert_eq!(first_block.span().end.column, 40);

        let second_block = splitter.next().unwrap();
        assert_eq!(
            second_block.src(),
            r#"This is a second paragraph.
It contains of three lines and sentences.
The paragraph is finalized by this sentence."#
        );
        assert_eq!(second_block.span().start.line, 4);
        assert_eq!(second_block.span().start.column, 1);
        assert_eq!(second_block.span().end.line, 6);
        assert_eq!(second_block.span().end.column, 45);

        assert!(splitter.next().is_none());
    }

    #[test]
    fn empty_source() {
        let src = "";

        let mut splitter = BlockSplitter::new(Box::new(src.as_bytes()));

        assert!(splitter.next().is_none());
    }

    #[test]
    fn ignore_consecutive_empty_lines() {
        let src = r#"Somewhere



over


the

rainbow!"#;

        let mut splitter = BlockSplitter::new(Box::new(src.as_bytes()));

        let block = splitter.next().unwrap();
        assert_eq!(block.src(), "Somewhere");
        assert_eq!(block.span().start.line, 1);
        assert_eq!(block.span().start.column, 1);
        assert_eq!(block.span().end.line, 1);
        assert_eq!(block.span().end.column, 10);

        let block = splitter.next().unwrap();
        assert_eq!(block.src(), "over");
        assert_eq!(block.span().start.line, 5);
        assert_eq!(block.span().start.column, 1);
        assert_eq!(block.span().end.line, 5);
        assert_eq!(block.span().end.column, 5);

        let block = splitter.next().unwrap();
        assert_eq!(block.src(), "the");
        assert_eq!(block.span().start.line, 8);
        assert_eq!(block.span().start.column, 1);
        assert_eq!(block.span().end.line, 8);
        assert_eq!(block.span().end.column, 4);

        let block = splitter.next().unwrap();
        assert_eq!(block.src(), "rainbow!");
        assert_eq!(block.span().start.line, 10);
        assert_eq!(block.span().start.column, 1);
        assert_eq!(block.span().end.line, 10);
        assert_eq!(block.span().end.column, 9);

        assert!(splitter.next().is_none());
    }

    #[test]
    fn allow_empty_lines_in_code_block() {
        let src = r#"```
const test = "test";

console.log(test);
```"#;

        let mut splitter = BlockSplitter::new(Box::new(src.as_bytes()));

        let block = splitter.next().unwrap();
        assert_eq!(
            block.src(),
            r#"```
const test = "test";

console.log(test);
```"#
        );
        assert_eq!(block.span().start.line, 1);
        assert_eq!(block.span().start.column, 1);
        assert_eq!(block.span().end.line, 5);
        assert_eq!(block.span().end.column, 4);

        assert!(splitter.next().is_none());
    }
}
