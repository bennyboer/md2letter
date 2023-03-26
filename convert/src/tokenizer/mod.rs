use std::collections::VecDeque;
use std::io::{BufReader, Read};

use utf8_chars::BufReadCharsExt;

use TokenKind::{BlockSeparator, HeadingLevel};

use crate::source_position::SourcePosition;
use crate::source_span::SourceSpan;
use crate::tokenizer::result::{TokenizeError, TokenizeResult};
use crate::tokenizer::token::TokenKind::{
    BoldEnd, BoldStart, CodeEnd, CodeStart, HorizontalRule, ItalicEnd, ItalicStart, Text,
};
use crate::tokenizer::token::{Token, TokenKind};

mod result;
mod token;

pub(crate) struct Tokenizer {
    reader: BufReader<Box<dyn Read>>,
    look_ahead_buffer: VecDeque<char>,
    offset: usize,
    ignore_updating_offset: usize,
    cursor_source_position: SourcePosition,
    last_char_source_position: SourcePosition,
    last_token_kind: Option<TokenKind>,
    future_closing_formatting_tokens: Vec<FutureToken>,
    is_in_code_emphasis: bool,
    next_token_is_code_emphasis: bool,
}

#[derive(Debug, Clone)]
struct FutureToken {
    token_kind: TokenKind,
    offset: usize,
}

impl Tokenizer {
    pub(crate) fn new(reader: Box<dyn Read>) -> Self {
        Self {
            reader: BufReader::new(Box::new(reader)),
            look_ahead_buffer: VecDeque::new(),
            offset: 0,
            ignore_updating_offset: 0,
            cursor_source_position: SourcePosition::zero(),
            last_char_source_position: SourcePosition::zero(),
            last_token_kind: None,
            future_closing_formatting_tokens: Vec::new(),
            is_in_code_emphasis: false,
            next_token_is_code_emphasis: false,
        }
    }

    pub fn next_char_source_position(&self) -> SourcePosition {
        self.cursor_source_position.clone()
    }

    pub fn last_char_source_position(&self) -> SourcePosition {
        self.last_char_source_position.clone()
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    fn read_next(&mut self) -> Option<char> {
        let next_char = if let Some(c) = self.look_ahead_buffer.pop_front() {
            Some(c)
        } else {
            self.reader.read_char().ok().flatten()
        };

        if let Some(c) = next_char {
            if c == '\r' {
                return self.read_next();
            }

            if self.ignore_updating_offset == 0 {
                self.last_char_source_position = self.cursor_source_position.clone();

                if c == '\n' {
                    self.cursor_source_position.line += 1;
                    self.cursor_source_position.column = 1;
                } else {
                    self.cursor_source_position.column += 1;
                }

                self.offset += 1;
            } else {
                self.ignore_updating_offset -= 1;
            }
        }

        next_char
    }

    fn look_ahead(&mut self, count: usize) -> Option<&[char]> {
        while self.look_ahead_buffer.len() < count {
            if let Some(c) = self.reader.read_char().ok().flatten() {
                self.look_ahead_buffer.push_back(c);
            } else {
                break;
            }
        }

        if self.look_ahead_buffer.len() >= count {
            let slice = self.look_ahead_buffer.make_contiguous();
            Some(&slice[..count])
        } else {
            None
        }
    }

    fn mark_char_as_unconsumed(&mut self, c: char) {
        self.look_ahead_buffer.push_front(c);
        self.ignore_updating_offset += 1;
    }

    fn is_at_block_start(&self) -> bool {
        match self.last_token_kind {
            Some(BlockSeparator) => true,
            None => true,
            _ => false,
        }
    }

    fn is_next_char_alphabetical(&mut self) -> bool {
        self.look_ahead(1)
            .map(|chars| chars[0].is_alphabetic())
            .unwrap_or(false)
    }

    fn find_reoccurring_char_count(&mut self, c: char) -> usize {
        let mut count = 0;
        while let Some(chars) = self.look_ahead(count + 1) {
            let next_char = *chars.last().unwrap();
            if next_char == c {
                count += 1;
            } else {
                break;
            }
        }

        count
    }

    fn next_char(&mut self) -> char {
        self.look_ahead(1).map(|chars| chars[0]).unwrap_or('\0')
    }

    fn ignore_next_chars(&mut self, count: usize) {
        for _ in 0..count {
            self.read_next();
        }
    }

    fn next_newline_is_only_separated_by_space(&mut self) -> (bool, usize) {
        let mut count = 0;

        while let Some(chars) = self.look_ahead(count + 1) {
            let next_char = *chars.last().unwrap();
            match next_char {
                ' ' | '\t' | '\r' => {}
                '\n' => return (true, count),
                _ => return (false, count),
            }

            count += 1;
        }

        (false, count)
    }

    fn find_next_char_matching(&mut self, c: char, start_at: usize) -> Option<usize> {
        let mut count = start_at;
        while let Some(chars) = self.look_ahead(count + 1) {
            let next_char = *chars.last().unwrap();
            if next_char == c {
                return Some(count);
            }

            count += 1;
        }

        None
    }

    fn find_formatting_pair(&mut self) -> Option<Vec<FutureToken>> {
        let mut is_italic = true;
        let mut is_bold = false;
        let mut in_code_emphasis = false;

        let mut result = Vec::new();

        if let Some(chars) = self.look_ahead(2) {
            if chars[0] == '*' {
                is_bold = true;

                if chars[1] != '*' {
                    is_italic = false;
                }
            }
        } else {
            return None;
        }

        #[derive(Debug)]
        enum Precedence {
            Bold,
            Italic,
            None,
        }

        let mut precedence = if is_bold && is_italic {
            Precedence::None
        } else if is_bold {
            result.push(FutureToken {
                token_kind: BoldStart,
                offset: self.offset(),
            });
            Precedence::Bold
        } else if is_italic {
            result.push(FutureToken {
                token_kind: ItalicStart,
                offset: self.offset(),
            });
            Precedence::Italic
        } else {
            unreachable!()
        };

        let mut count = 2;
        let mut ignore_next_star = false;
        while let Some(chars) = self.look_ahead(count + 1) {
            let next_char = *chars.last().unwrap();

            if in_code_emphasis {
                if next_char == '`' {
                    in_code_emphasis = false;
                }
                count += 1;
                continue;
            }

            match next_char {
                '\\' => ignore_next_star = true,
                '`' => {
                    let count = self.find_next_char_matching('`', count + 1);
                    if let Some(_) = count {
                        in_code_emphasis = true;
                    }
                }
                '*' => {
                    if ignore_next_star {
                        ignore_next_star = false;
                        count += 1;
                        continue;
                    }

                    if is_italic && is_bold {
                        // Cannot be another "more formatting" than there already is -> must be a closing star
                        if let Some(chars) = self.look_ahead(count + 3) {
                            let char_after_next = chars[chars.len() - 2];
                            let last_char = chars[chars.len() - 1];
                            if char_after_next == '*' {
                                is_bold = false;
                                count += 1; // Ignore next star

                                if last_char == '*' {
                                    match precedence {
                                        Precedence::Bold => {
                                            result.push(FutureToken {
                                                token_kind: ItalicEnd,
                                                offset: self.offset() + count,
                                            });
                                            result.push(FutureToken {
                                                token_kind: BoldEnd,
                                                offset: self.offset() + count + 1,
                                            });
                                        }
                                        Precedence::Italic => {
                                            result.push(FutureToken {
                                                token_kind: BoldEnd,
                                                offset: self.offset() + count + 1,
                                            });
                                            result.push(FutureToken {
                                                token_kind: ItalicEnd,
                                                offset: self.offset() + count + 2,
                                            });
                                        }
                                        Precedence::None => {
                                            result.push(FutureToken {
                                                token_kind: BoldEnd,
                                                offset: self.offset() + count + 1,
                                            });
                                            result.insert(
                                                0,
                                                FutureToken {
                                                    token_kind: ItalicStart,
                                                    offset: self.offset(),
                                                },
                                            );
                                            result.push(FutureToken {
                                                token_kind: ItalicEnd,
                                                offset: self.offset() + count + 2,
                                            });
                                        }
                                    };

                                    return Some(result);
                                }

                                result.push(FutureToken {
                                    token_kind: BoldEnd,
                                    offset: self.offset() + count + 1,
                                });
                                if let Precedence::None = precedence {
                                    result.insert(
                                        0,
                                        FutureToken {
                                            token_kind: ItalicStart,
                                            offset: self.offset(),
                                        },
                                    );
                                    result.insert(
                                        1,
                                        FutureToken {
                                            token_kind: BoldStart,
                                            offset: self.offset() + 2,
                                        },
                                    );
                                    precedence = Precedence::Italic;
                                }
                            } else {
                                is_italic = false;

                                result.push(FutureToken {
                                    token_kind: ItalicEnd,
                                    offset: self.offset() + count + 1,
                                });
                                if let Precedence::None = precedence {
                                    result.insert(
                                        0,
                                        FutureToken {
                                            token_kind: BoldStart,
                                            offset: self.offset(),
                                        },
                                    );
                                    result.insert(
                                        1,
                                        FutureToken {
                                            token_kind: ItalicStart,
                                            offset: self.offset() + 2,
                                        },
                                    );
                                    precedence = Precedence::Bold;
                                }
                            }
                        }
                    } else if is_italic {
                        // Must be a closing star or another bold opening
                        let is_bold_opening = if let Some(chars) = self.look_ahead(count + 2) {
                            let char_after_next = *chars.last().unwrap();
                            char_after_next == '*'
                        } else {
                            false
                        };

                        if is_bold_opening {
                            is_bold = true;
                            count += 1; // Ignore next star

                            result.push(FutureToken {
                                token_kind: BoldStart,
                                offset: self.offset() + count + 1,
                            });
                        } else {
                            // Found a italic closing -> formatting pair is italic
                            result.push(FutureToken {
                                token_kind: ItalicEnd,
                                offset: self.offset() + count + 1,
                            });
                            return Some(result);
                        }
                    } else if is_bold {
                        // Must be a bold closing or another italic opening
                        let is_italic_opening = if let Some(chars) = self.look_ahead(count + 2) {
                            let char_after_next = *chars.last().unwrap();
                            char_after_next != '*'
                        } else {
                            true
                        };

                        if is_italic_opening {
                            is_italic = true;
                            result.push(FutureToken {
                                token_kind: ItalicStart,
                                offset: self.offset() + count + 1,
                            });
                        } else {
                            // Found a bold closing -> formatting pair is bold
                            result.push(FutureToken {
                                token_kind: BoldEnd,
                                offset: self.offset() + count + 1,
                            });
                            return Some(result);
                        }
                    }
                }
                _ => {
                    if ignore_next_star {
                        ignore_next_star = false;
                    }
                }
            }

            count += 1;
        }

        None
    }

    fn consume_text_buffer_and_create_text_token(
        &mut self,
        start_position: &SourcePosition,
        text_buffer: &String,
    ) -> Option<TokenizeResult<Token>> {
        return Some(Ok(Token::new(
            Text(text_buffer.to_string()),
            SourceSpan::new(start_position.clone(), self.next_char_source_position()),
        )));
    }
}

impl Iterator for Tokenizer {
    type Item = TokenizeResult<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        let start_position = self.next_char_source_position();
        let mut text_buffer = String::new();
        let mut treat_next_special_char_as_text = false;

        loop {
            let next_char = self.read_next();
            match next_char {
                Some(c) => {
                    println!("Next char: '{}' | {}", c, self.is_in_code_emphasis);

                    if treat_next_special_char_as_text {
                        treat_next_special_char_as_text = false;
                        text_buffer.push(c);
                        continue;
                    }

                    if self.is_in_code_emphasis {
                        if self.next_token_is_code_emphasis {
                            self.next_token_is_code_emphasis = false;
                            self.is_in_code_emphasis = false;
                            return Some(Ok(Token::new(
                                CodeEnd,
                                SourceSpan::new(
                                    start_position.clone(),
                                    self.next_char_source_position(),
                                ),
                            )));
                        }

                        if c == '`' {
                            self.next_token_is_code_emphasis = true;
                            self.mark_char_as_unconsumed('`');
                            return self.consume_text_buffer_and_create_text_token(
                                &start_position,
                                &text_buffer,
                            );
                        } else {
                            text_buffer.push(c);
                        }
                        continue;
                    }

                    match c {
                        '\\' => {
                            treat_next_special_char_as_text = true;
                        }
                        '#' => {
                            let is_at_block_start = self.is_at_block_start();
                            let is_function_start = self.is_next_char_alphabetical();

                            let is_heading = is_at_block_start && !is_function_start;
                            if is_heading {
                                let level = self.find_reoccurring_char_count('#');
                                self.ignore_next_chars(level);

                                return Some(Ok(Token::new(
                                    HeadingLevel(level),
                                    SourceSpan::new(
                                        start_position,
                                        self.last_char_source_position(),
                                    ),
                                )));
                            }
                        }
                        ' ' | '\t' => {
                            text_buffer.push(' ');
                        }
                        '\n' => {
                            let (is_block_separator, char_count_until_newline) =
                                self.next_newline_is_only_separated_by_space();
                            if is_block_separator {
                                if !text_buffer.is_empty() {
                                    self.mark_char_as_unconsumed('\n');
                                    return self.consume_text_buffer_and_create_text_token(
                                        &start_position,
                                        &text_buffer,
                                    );
                                }

                                self.ignore_next_chars(char_count_until_newline + 1);

                                return Some(Ok(Token::new(
                                    BlockSeparator,
                                    SourceSpan::new(
                                        start_position,
                                        self.next_char_source_position(),
                                    ),
                                )));
                            }

                            text_buffer.push(' ');
                        }
                        '_' => {
                            let is_at_block_start = self.is_at_block_start();
                            if is_at_block_start {
                                let reoccuring_char_count = self.find_reoccurring_char_count('_');
                                let is_horizontal_rule = reoccuring_char_count >= 2;
                                if is_horizontal_rule {
                                    self.ignore_next_chars(reoccuring_char_count);
                                    return Some(Ok(Token::new(
                                        HorizontalRule,
                                        SourceSpan::new(
                                            start_position,
                                            self.next_char_source_position(),
                                        ),
                                    )));
                                }
                            }

                            text_buffer.push('_');
                        }
                        '+' => {
                            let is_at_block_start = self.is_at_block_start();
                            if is_at_block_start {
                                let reoccuring_char_count = self.find_reoccurring_char_count('+');
                                let is_horizontal_rule = reoccuring_char_count >= 2;
                                if is_horizontal_rule {
                                    self.ignore_next_chars(reoccuring_char_count);
                                    return Some(Ok(Token::new(
                                        HorizontalRule,
                                        SourceSpan::new(
                                            start_position,
                                            self.next_char_source_position(),
                                        ),
                                    )));
                                }

                                let is_list_item =
                                    reoccuring_char_count == 0 && self.next_char() == ' ';
                                if is_list_item {
                                    todo!();
                                }
                            }

                            text_buffer.push('+');
                        }
                        '-' => {
                            let is_at_block_start = self.is_at_block_start();
                            if is_at_block_start {
                                let reoccuring_char_count = self.find_reoccurring_char_count('-');
                                let is_horizontal_rule = reoccuring_char_count >= 2;
                                if is_horizontal_rule {
                                    self.ignore_next_chars(reoccuring_char_count);
                                    return Some(Ok(Token::new(
                                        HorizontalRule,
                                        SourceSpan::new(
                                            start_position,
                                            self.next_char_source_position(),
                                        ),
                                    )));
                                }

                                let is_list_item =
                                    reoccuring_char_count == 0 && self.next_char() == ' ';
                                if is_list_item {
                                    todo!();
                                }
                            }

                            text_buffer.push('-');
                        }
                        '*' => {
                            let offset = self.offset();
                            let future_closing_formatting_token = self
                                .future_closing_formatting_tokens
                                .iter()
                                .find(|token| token.offset == offset);
                            if let Some(t) = future_closing_formatting_token.cloned() {
                                if !text_buffer.is_empty() {
                                    self.mark_char_as_unconsumed('*');
                                    return self.consume_text_buffer_and_create_text_token(
                                        &start_position,
                                        &text_buffer,
                                    );
                                }

                                self.future_closing_formatting_tokens
                                    .retain(|t| t.offset != offset);

                                let is_bold = t.token_kind == BoldEnd;
                                let star_count = if is_bold { 2 } else { 1 };
                                if is_bold {
                                    self.ignore_next_chars(1);
                                }

                                let next_char_source_position = self.next_char_source_position();
                                return Some(Ok(Token::new(
                                    t.token_kind.clone(),
                                    SourceSpan::new(
                                        start_position,
                                        SourcePosition {
                                            line: next_char_source_position.line,
                                            column: next_char_source_position.column + star_count
                                                - 1,
                                        },
                                    ),
                                )));
                            }

                            let is_list_item = false; // TODO implement list items
                            if is_list_item {
                                todo!()
                            } else {
                                if let Some(future_tokens) = self.find_formatting_pair() {
                                    if !text_buffer.is_empty() {
                                        self.mark_char_as_unconsumed('*');
                                        return self.consume_text_buffer_and_create_text_token(
                                            &start_position,
                                            &text_buffer,
                                        );
                                    }

                                    println!("Found tokens: {:?}", future_tokens);

                                    let is_bold = future_tokens[0].token_kind == BoldStart;
                                    for token in &future_tokens[1..] {
                                        self.future_closing_formatting_tokens.push(token.clone());
                                    }

                                    return if is_bold {
                                        self.ignore_next_chars(1);

                                        Some(Ok(Token::new(
                                            BoldStart,
                                            SourceSpan::new(
                                                start_position,
                                                self.next_char_source_position(),
                                            ),
                                        )))
                                    } else {
                                        Some(Ok(Token::new(
                                            ItalicStart,
                                            SourceSpan::new(
                                                start_position,
                                                self.next_char_source_position(),
                                            ),
                                        )))
                                    };
                                } else {
                                    text_buffer.push(c);
                                }
                            }
                        }
                        '`' => {
                            if !text_buffer.is_empty() {
                                self.mark_char_as_unconsumed('`');
                                return self.consume_text_buffer_and_create_text_token(
                                    &start_position,
                                    &mut text_buffer,
                                );
                            }

                            let closing_char_offset = self.find_next_char_matching('`', 0);
                            match closing_char_offset {
                                None => {
                                    return Some(Err(TokenizeError {
                                        message: "Could not find closing backtick".to_string(),
                                        source_position: start_position,
                                    }))
                                }
                                Some(_) => {
                                    self.is_in_code_emphasis = true;
                                }
                            }

                            return Some(Ok(Token::new(
                                CodeStart,
                                SourceSpan::new(start_position, self.next_char_source_position()),
                            )));
                        }
                        '\r' => {}
                        _ => text_buffer.push(c),
                    }
                }
                None => {
                    if text_buffer.is_empty() {
                        return None;
                    }

                    return Some(Ok(Token::new(
                        Text(text_buffer),
                        SourceSpan::new(start_position, self.next_char_source_position()),
                    )));
                }
            };
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use crate::source_span::SourceSpan;
    use crate::tokenizer::token::TokenKind::{
        BlockSeparator, BoldEnd, BoldStart, CodeEnd, CodeStart, HeadingLevel, HorizontalRule,
        ItalicEnd, ItalicStart, ListItemLevel, Text,
    };

    use super::*;

    // TODO Table
    // TODO Link
    // TODO Functions
    // TODO Images
    // TODO Lists

    #[test]
    fn tokenize_heading() {
        let src = "# This is a heading";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, HeadingLevel(0));
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" This is a heading".to_string())
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_heading_with_level() {
        let src = "##### This is a heading";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, HeadingLevel(4));
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" This is a heading".to_string())
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn should_tokenize_as_text_and_not_as_horizontal_rule() {
        let src = "--";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("--".to_string())
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_horizontal_rule_with_minus_char() {
        let src = "---";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, HorizontalRule);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_horizontal_rule_with_plus_char() {
        let src = "+++";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, HorizontalRule);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_horizontal_rule_with_underline_char() {
        let src = "___";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, HorizontalRule);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_horizontal_rule_with_more_than_necessary_chars() {
        let src = "--------------------------";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, HorizontalRule);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_horizontal_rule_complex() {
        let src = "# Heading

---

And some text.";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, HeadingLevel(0));
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" Heading".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BlockSeparator);
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, HorizontalRule);
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BlockSeparator);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("And some text.".to_string())
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_block_separator() {
        let src = r#"Paragraph

Paragraph"#;

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
    }

    #[test]
    fn tokenize_italic_emphasis() {
        let src = "*This is emphasized*";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("This is emphasized".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_emphasis() {
        let src = "**This is emphasized**";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("This is emphasized".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_mixed_bold_and_italic_emphasis_italic_first() {
        let src = "*This is emphasized **some** way*";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("This is emphasized ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("some".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" way".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_mixed_bold_and_italic_emphasis_bold_first() {
        let src = "**This is emphasized *some* way**";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("This is emphasized ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("some".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" way".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic() {
        let src = "***THIS IS TEXT***";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("THIS IS TEXT".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_2() {
        let src = "***THIS IS TEXT**Hello World*";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("THIS IS TEXT".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Hello World".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_3() {
        let src = "*Hello World**THIS IS TEXT***";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Hello World".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("THIS IS TEXT".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_4() {
        let src = "**Hello World*THIS IS TEXT***";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Hello World".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("THIS IS TEXT".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_5() {
        let src = "***THIS IS TEXT*Hello World**";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("THIS IS TEXT".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Hello World".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_6() {
        let src = "Here are *some **stars** for you*";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Here are ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("some ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("stars".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" for you".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_7() {
        let src = "Here are **some *stars* for you**";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Here are ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("some ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("stars".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" for you".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_escaped() {
        let src = "**There is a \\* star**";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("There is a * star".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_code_emphasis_trivial() {
        let src = "`This is emphasized`";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(tokenizer.next().unwrap().unwrap().kind, CodeStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("This is emphasized".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, CodeEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_code_emphasis_in_context() {
        let src = "Here is some `emphasized` text";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Here is some ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, CodeStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("emphasized".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, CodeEnd);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" text".to_string())
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_code_emphasis_and_ignore_special_chars() {
        let src = r#"In `*this*` case, the \* should be ignored"#;

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("In ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, CodeStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("*this*".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, CodeEnd);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" case, the * should be ignored".to_string())
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_code_emphasis_and_ignore_formatting() {
        let src = "We have some *formatting `*code*`* and in the middle is `code`";

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("We have some ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("formatting ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, CodeStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("*code*".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, CodeEnd);
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" and in the middle is ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, CodeStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("code".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, CodeEnd);
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_unordered_list_item_level() {
        let src = r#"
- Item 1
    + Item 2
    * Item 3
1. Item 4
  2. Item 5
    3. Item 6
      4. Item 7
"#;

        let reader = Box::new(BufReader::new(src.as_bytes()));
        let mut tokenizer = Tokenizer::new(reader);

        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            ListItemLevel {
                level: 0,
                ordered: false
            }
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Item 1".to_string())
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            ListItemLevel {
                level: 1,
                ordered: false
            }
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Item 2".to_string())
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            ListItemLevel {
                level: 1,
                ordered: false
            }
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Item 3".to_string())
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            ListItemLevel {
                level: 0,
                ordered: true
            }
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Item 4".to_string())
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            ListItemLevel {
                level: 1,
                ordered: true
            }
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Item 5".to_string())
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            ListItemLevel {
                level: 2,
                ordered: true
            }
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Item 6".to_string())
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            ListItemLevel {
                level: 3,
                ordered: true
            }
        );
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("Item 7".to_string())
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_complex() {
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
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("heading".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, ItalicEnd);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text(" with emphasis".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BlockSeparator);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("This is a paragraph with ".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldStart);
        assert_eq!(
            tokenizer.next().unwrap().unwrap().kind,
            Text("bold".to_string())
        );
        assert_eq!(tokenizer.next().unwrap().unwrap().kind, BoldEnd);
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
    }

    #[test]
    fn should_include_source_span_in_tokens() {
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
                    start: SourcePosition {
                        line: 1,
                        column: 16
                    },
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
                BoldStart,
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
                BoldEnd,
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
    }
}
