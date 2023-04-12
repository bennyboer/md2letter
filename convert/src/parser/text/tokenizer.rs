use std::collections::{HashMap, VecDeque};

use crate::parser::text::token::TokenKind::{
    BoldEnd, BoldStart, CodeEnd, CodeStart, Error, Function, Image, ItalicEnd, ItalicStart, Link,
    Text,
};
use crate::parser::text::token::{Token, TokenKind};
use crate::util::{SourcePosition, SourceSpan};

const MAX_SOURCE_POSITION_UPDATE_HISTORY_SIZE: usize = 100;

pub(crate) struct Tokenizer {
    src: String,
    offset: usize,
    offset_source_position: SourcePosition,
    source_position_update_history: VecDeque<SourcePositionUpdate>,
    is_in_code_emphasis: bool,
    next_token_is_code_emphasis: bool,
    future_closing_formatting_tokens: Vec<FutureToken>,
    is_initialized: bool,
}

enum SourcePositionUpdate {
    NewLine { old_column: usize },
    Column,
    Ignore,
}

#[derive(Debug, Clone)]
struct FutureToken {
    token_kind: TokenKind,
    offset: usize,
}

impl Tokenizer {
    pub(crate) fn new(src: String, span: SourceSpan) -> Self {
        Self {
            src,
            offset: 0,
            offset_source_position: span.start,
            source_position_update_history: VecDeque::new(),
            is_in_code_emphasis: false,
            next_token_is_code_emphasis: false,
            future_closing_formatting_tokens: Vec::new(),
            is_initialized: false,
        }
    }

    fn offset_source_position(&self) -> SourcePosition {
        self.offset_source_position.clone()
    }

    fn read_at(&self, offset: usize) -> Option<char> {
        self.src.chars().nth(offset)
    }

    fn read_next(&mut self) -> Option<char> {
        if !self.is_initialized {
            self.is_initialized = true;
        } else {
            self.offset += 1;
        }

        let next_char = self.read_at(self.offset);

        if let Some(c) = next_char {
            if c == '\r' {
                self.source_position_update_history
                    .push_front(SourcePositionUpdate::Ignore);
                return self.read_next();
            }

            if c == '\n' {
                self.source_position_update_history
                    .push_front(SourcePositionUpdate::NewLine {
                        old_column: self.offset_source_position.column,
                    });

                self.offset_source_position.line += 1;
                self.offset_source_position.column = 1;
            } else {
                self.source_position_update_history
                    .push_front(SourcePositionUpdate::Column);

                self.offset_source_position.column += 1;
            }

            if self.source_position_update_history.len() > MAX_SOURCE_POSITION_UPDATE_HISTORY_SIZE {
                self.source_position_update_history.pop_back();
            }
        }

        next_char
    }

    fn mark_char_as_unconsumed(&mut self) {
        self.offset -= 1;

        let update = self.source_position_update_history.pop_front();
        if let Some(update) = update {
            match update {
                SourcePositionUpdate::NewLine { old_column } => {
                    self.offset_source_position.line -= 1;
                    self.offset_source_position.column = old_column;
                }
                SourcePositionUpdate::Column => {
                    self.offset_source_position.column -= 1;
                }
                _ => {}
            }
        } else {
            panic!(
                "Tried to mark char as unconsumed, but there is no update in history to revert."
            );
        }
    }

    fn ignore_next_chars(&mut self, count: usize) {
        for _ in 0..count {
            self.read_next();
        }
    }

    fn look_ahead(&self, count: usize) -> Option<char> {
        self.read_at(self.offset + count)
    }

    fn find_next_char_matching(&mut self, c: char, start_at: usize) -> Option<usize> {
        let mut count = start_at;
        while let Some(next_char) = self.look_ahead(count + 1) {
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

        let next_char_1 = self.look_ahead(1);
        let next_char_2 = self.look_ahead(2);
        if next_char_1.is_none() || next_char_2.is_none() {
            return None;
        }
        if next_char_1 == Some('*') {
            is_bold = true;

            if let Some(last_char) = next_char_2 {
                if last_char != '*' {
                    is_italic = false;
                }
            }
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
                offset: self.offset,
            });
            Precedence::Bold
        } else if is_italic {
            result.push(FutureToken {
                token_kind: ItalicStart,
                offset: self.offset,
            });
            Precedence::Italic
        } else {
            unreachable!()
        };

        let mut count = 2;
        let mut ignore_next_star = false;
        while let Some(next_char) = self.look_ahead(count + 1) {
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
                        let next_char_1 = self.look_ahead(count + 2);
                        let next_char_2 = self.look_ahead(count + 3);

                        if let Some('*') = next_char_1 {
                            is_bold = false;
                            count += 1; // Ignore next star

                            if let Some('*') = next_char_2 {
                                match precedence {
                                    Precedence::Bold => {
                                        result.push(FutureToken {
                                            token_kind: ItalicEnd,
                                            offset: self.offset + count,
                                        });
                                        result.push(FutureToken {
                                            token_kind: BoldEnd,
                                            offset: self.offset + count + 1,
                                        });
                                    }
                                    Precedence::Italic => {
                                        result.push(FutureToken {
                                            token_kind: BoldEnd,
                                            offset: self.offset + count + 1,
                                        });
                                        result.push(FutureToken {
                                            token_kind: ItalicEnd,
                                            offset: self.offset + count + 2,
                                        });
                                    }
                                    Precedence::None => {
                                        result.push(FutureToken {
                                            token_kind: BoldEnd,
                                            offset: self.offset + count + 1,
                                        });
                                        result.insert(
                                            0,
                                            FutureToken {
                                                token_kind: ItalicStart,
                                                offset: self.offset,
                                            },
                                        );
                                        result.push(FutureToken {
                                            token_kind: ItalicEnd,
                                            offset: self.offset + count + 2,
                                        });
                                    }
                                };

                                return Some(result);
                            }

                            result.push(FutureToken {
                                token_kind: BoldEnd,
                                offset: self.offset + count,
                            });
                            if let Precedence::None = precedence {
                                result.insert(
                                    0,
                                    FutureToken {
                                        token_kind: ItalicStart,
                                        offset: self.offset,
                                    },
                                );
                                result.insert(
                                    1,
                                    FutureToken {
                                        token_kind: BoldStart,
                                        offset: self.offset + 2,
                                    },
                                );
                                precedence = Precedence::Italic;
                            }
                        } else {
                            is_italic = false;

                            result.push(FutureToken {
                                token_kind: ItalicEnd,
                                offset: self.offset + count + 1,
                            });
                            if let Precedence::None = precedence {
                                result.insert(
                                    0,
                                    FutureToken {
                                        token_kind: BoldStart,
                                        offset: self.offset,
                                    },
                                );
                                result.insert(
                                    1,
                                    FutureToken {
                                        token_kind: ItalicStart,
                                        offset: self.offset + 2,
                                    },
                                );
                                precedence = Precedence::Bold;
                            }
                        }
                    } else if is_italic {
                        // Must be a closing star or another bold opening
                        let is_bold_opening =
                            if let Some(char_after_next) = self.look_ahead(count + 2) {
                                char_after_next == '*'
                            } else {
                                false
                            };

                        if is_bold_opening {
                            is_bold = true;
                            count += 1; // Ignore next star

                            result.push(FutureToken {
                                token_kind: BoldStart,
                                offset: self.offset + count + 1,
                            });
                        } else {
                            // Found a italic closing -> formatting pair is italic
                            result.push(FutureToken {
                                token_kind: ItalicEnd,
                                offset: self.offset + count + 1,
                            });
                            return Some(result);
                        }
                    } else if is_bold {
                        // Must be a bold closing or another italic opening
                        let is_italic_opening =
                            if let Some(char_after_next) = self.look_ahead(count + 2) {
                                char_after_next != '*'
                            } else {
                                true
                            };

                        if is_italic_opening {
                            is_italic = true;
                            result.push(FutureToken {
                                token_kind: ItalicStart,
                                offset: self.offset + count + 1,
                            });
                        } else {
                            // Found a bold closing -> formatting pair is bold
                            result.push(FutureToken {
                                token_kind: BoldEnd,
                                offset: self.offset + count + 1,
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
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let start_position = self.offset_source_position();
        let mut text_buffer = String::new();
        let mut treat_next_special_char_as_text = false;

        loop {
            let next_char = self.read_next();
            match next_char {
                Some(c) => {
                    if treat_next_special_char_as_text {
                        treat_next_special_char_as_text = false;
                        text_buffer.push(c);
                        continue;
                    }

                    if self.is_in_code_emphasis {
                        if self.next_token_is_code_emphasis {
                            self.next_token_is_code_emphasis = false;
                            self.is_in_code_emphasis = false;

                            return Some(Token::new(
                                CodeEnd,
                                SourceSpan::new(
                                    start_position.clone(),
                                    self.offset_source_position().clone(),
                                ),
                            ));
                        }

                        if c == '`' {
                            self.next_token_is_code_emphasis = true;
                            self.mark_char_as_unconsumed();
                            return Some(Token::new(
                                Text(text_buffer.to_string()),
                                SourceSpan::new(
                                    start_position.clone(),
                                    self.offset_source_position().clone(),
                                ),
                            ));
                        } else {
                            text_buffer.push(c);
                        }
                        continue;
                    }

                    match c {
                        '\\' => {
                            treat_next_special_char_as_text = true;
                        }
                        ' ' | '\t' => {
                            text_buffer.push(' ');
                        }
                        '\n' => {
                            if text_buffer.chars().last() != Some(' ') {
                                text_buffer.push(' ');
                            }
                        }
                        '#' => {
                            let mut count = 1;
                            let mut function_name = String::new();
                            loop {
                                if let Some(next_char) = self.look_ahead(count) {
                                    if next_char == '(' {
                                        break;
                                    } else {
                                        function_name.push(next_char);
                                    }
                                } else {
                                    break;
                                }

                                count += 1;
                            }

                            if function_name.is_empty() {
                                text_buffer.push(c);
                                continue;
                            }

                            count += 1;
                            let mut parameters: HashMap<String, String> = HashMap::new();
                            let mut parameter_name = String::new();
                            let mut parameter_value = String::new();
                            let mut is_in_arg_name = true;

                            loop {
                                if let Some(next_char) = self.look_ahead(count) {
                                    match next_char {
                                        ')' => {
                                            if !parameter_name.trim().is_empty() {
                                                parameters.insert(
                                                    parameter_name.trim().to_string(),
                                                    parameter_value.trim().to_string(),
                                                );
                                            }
                                            break;
                                        }
                                        ',' => {
                                            if !parameter_value.trim().is_empty() {
                                                parameters.insert(
                                                    parameter_name.trim().to_string(),
                                                    parameter_value.trim().to_string(),
                                                );
                                            }
                                            parameter_name.clear();
                                            parameter_value.clear();
                                            is_in_arg_name = true;
                                        }
                                        ':' => {
                                            if is_in_arg_name {
                                                is_in_arg_name = false;
                                            } else {
                                                parameter_value.push(next_char);
                                            }
                                        }
                                        _ => {
                                            if is_in_arg_name {
                                                parameter_name.push(next_char);
                                            } else {
                                                parameter_value.push(next_char);
                                            }
                                        }
                                    }
                                } else {
                                    break;
                                }

                                count += 1;
                            }

                            if !text_buffer.is_empty() {
                                self.mark_char_as_unconsumed();
                                return Some(Token::new(
                                    Text(text_buffer.to_string()),
                                    SourceSpan::new(
                                        start_position.clone(),
                                        self.offset_source_position().clone(),
                                    ),
                                ));
                            }

                            self.ignore_next_chars(count);

                            return Some(Token::new(
                                Function {
                                    name: function_name,
                                    parameters,
                                },
                                SourceSpan::new(
                                    start_position.clone(),
                                    self.offset_source_position().clone(),
                                ),
                            ));
                        }
                        '!' => {
                            // Check if next char is '['
                            let may_be_image = if let Some('[') = self.look_ahead(1) {
                                true
                            } else {
                                false
                            };
                            if !may_be_image {
                                text_buffer.push(c);
                                continue;
                            }

                            // Find label
                            let mut count = 2;
                            let mut label = String::new();
                            loop {
                                if let Some(next_char) = self.look_ahead(count) {
                                    if next_char == ']' {
                                        break;
                                    } else {
                                        label.push(next_char);
                                    }
                                } else {
                                    break;
                                }

                                count += 1;
                            }

                            if label.is_empty() {
                                text_buffer.push(c);
                                continue;
                            }

                            // Find src
                            let is_opening_parenthesis = self.look_ahead(count + 1) == Some('(');
                            if !is_opening_parenthesis {
                                text_buffer.push(c);
                                continue;
                            }

                            count += 2;
                            let mut src = String::new();
                            loop {
                                if let Some(next_char) = self.look_ahead(count) {
                                    if next_char == ')' {
                                        break;
                                    } else {
                                        src.push(next_char);
                                    }
                                } else {
                                    break;
                                }

                                count += 1;
                            }

                            if src.is_empty() {
                                text_buffer.push(c);
                                continue;
                            }

                            if !text_buffer.is_empty() {
                                self.mark_char_as_unconsumed();

                                return Some(Token::new(
                                    Text(text_buffer.to_string()),
                                    SourceSpan::new(
                                        start_position.clone(),
                                        self.offset_source_position().clone(),
                                    ),
                                ));
                            }

                            self.ignore_next_chars(count);
                            return Some(Token::new(
                                Image { label, src },
                                SourceSpan::new(
                                    start_position.clone(),
                                    self.offset_source_position().clone(),
                                ),
                            ));
                        }
                        '[' => {
                            // Find label
                            let mut count = 1;
                            let mut label = String::new();
                            loop {
                                if let Some(next_char) = self.look_ahead(count) {
                                    if next_char == ']' {
                                        break;
                                    } else {
                                        label.push(next_char);
                                    }
                                } else {
                                    break;
                                }

                                count += 1;
                            }

                            if label.is_empty() {
                                text_buffer.push(c);
                                continue;
                            }

                            // Find target
                            let is_opening_parenthesis = self.look_ahead(count + 1) == Some('(');
                            if !is_opening_parenthesis {
                                text_buffer.push(c);
                                continue;
                            }

                            count += 2;
                            let mut target = String::new();
                            loop {
                                if let Some(next_char) = self.look_ahead(count) {
                                    if next_char == ')' {
                                        break;
                                    } else {
                                        target.push(next_char);
                                    }
                                } else {
                                    break;
                                }

                                count += 1;
                            }

                            if target.is_empty() {
                                text_buffer.push(c);
                                continue;
                            }

                            if !text_buffer.is_empty() {
                                self.mark_char_as_unconsumed();

                                return Some(Token::new(
                                    Text(text_buffer.to_string()),
                                    SourceSpan::new(
                                        start_position.clone(),
                                        self.offset_source_position().clone(),
                                    ),
                                ));
                            }

                            self.ignore_next_chars(count);
                            return Some(Token::new(
                                Link { label, target },
                                SourceSpan::new(
                                    start_position.clone(),
                                    self.offset_source_position().clone(),
                                ),
                            ));
                        }
                        '*' => {
                            let offset = self.offset;
                            let future_closing_formatting_token = self
                                .future_closing_formatting_tokens
                                .iter()
                                .find(|token| token.offset == offset);
                            if let Some(t) = future_closing_formatting_token.cloned() {
                                if !text_buffer.is_empty() {
                                    self.mark_char_as_unconsumed();

                                    return Some(Token::new(
                                        Text(text_buffer.to_string()),
                                        SourceSpan::new(
                                            start_position.clone(),
                                            self.offset_source_position().clone(),
                                        ),
                                    ));
                                }

                                self.future_closing_formatting_tokens
                                    .retain(|t| t.offset != offset);

                                let is_bold = t.token_kind == BoldEnd;
                                if is_bold {
                                    self.ignore_next_chars(1);
                                }

                                let next_char_source_position = self.offset_source_position();
                                return Some(Token::new(
                                    t.token_kind.clone(),
                                    SourceSpan::new(start_position, next_char_source_position),
                                ));
                            }

                            if let Some(future_tokens) = self.find_formatting_pair() {
                                if !text_buffer.is_empty() {
                                    self.mark_char_as_unconsumed();

                                    return Some(Token::new(
                                        Text(text_buffer.to_string()),
                                        SourceSpan::new(
                                            start_position.clone(),
                                            self.offset_source_position().clone(),
                                        ),
                                    ));
                                }

                                let is_bold = future_tokens[0].token_kind == BoldStart;
                                for token in &future_tokens[1..] {
                                    self.future_closing_formatting_tokens.push(token.clone());
                                }

                                return if is_bold {
                                    self.ignore_next_chars(1);

                                    Some(Token::new(
                                        BoldStart,
                                        SourceSpan::new(
                                            start_position,
                                            self.offset_source_position(),
                                        ),
                                    ))
                                } else {
                                    Some(Token::new(
                                        ItalicStart,
                                        SourceSpan::new(
                                            start_position,
                                            self.offset_source_position(),
                                        ),
                                    ))
                                };
                            } else {
                                text_buffer.push(c);
                            }
                        }
                        '`' => {
                            if !text_buffer.is_empty() {
                                self.mark_char_as_unconsumed();

                                return Some(Token::new(
                                    Text(text_buffer.to_string()),
                                    SourceSpan::new(
                                        start_position.clone(),
                                        self.offset_source_position().clone(),
                                    ),
                                ));
                            }

                            let closing_char_offset = self.find_next_char_matching('`', 0);
                            match closing_char_offset {
                                None => {
                                    return Some(Token::new(
                                        Error {
                                            message: "Could not find closing backtick".to_string(),
                                            source_position: self.offset_source_position(),
                                        },
                                        SourceSpan::new(
                                            start_position,
                                            self.offset_source_position(),
                                        ),
                                    ));
                                }
                                Some(_) => {
                                    self.is_in_code_emphasis = true;
                                }
                            }

                            return Some(Token::new(
                                CodeStart,
                                SourceSpan::new(start_position, self.offset_source_position()),
                            ));
                        }
                        '\r' => {}
                        _ => text_buffer.push(c),
                    }
                }
                None => {
                    if text_buffer.is_empty() {
                        return None;
                    }

                    return Some(Token::new(
                        Text(text_buffer),
                        SourceSpan::new(start_position, self.offset_source_position()),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::parser::text::token::TokenKind::{
        BoldEnd, BoldStart, CodeEnd, CodeStart, Function, Image, ItalicEnd, ItalicStart, Link, Text,
    };

    use super::*;

    #[test]
    fn tokenize_trivial() {
        let src = "This is a simple paragraph.";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 28)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("This is a simple paragraph.".to_string()),
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 28)),
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_trivial_over_multiple_lines() {
        let src = "This is a simple paragraph
that spans over multiple
lines.";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 7)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("This is a simple paragraph that spans over multiple lines.".to_string()),
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 7)),
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_italic_emphasis() {
        let src = "*This is emphasized*";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 21)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 2))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("This is emphasized".to_string()),
                SourceSpan::new(SourcePosition::new(1, 2), SourcePosition::new(1, 20))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicEnd,
                SourceSpan::new(SourcePosition::new(1, 20), SourcePosition::new(1, 21))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_emphasis() {
        let src = "**This is emphasized**";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 23)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 3))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("This is emphasized".to_string()),
                SourceSpan::new(SourcePosition::new(1, 3), SourcePosition::new(1, 21))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 21), SourcePosition::new(1, 23))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_mixed_bold_and_italic_emphasis_italic_first() {
        let src = "*This is emphasized **some** way*";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 34)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 2))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("This is emphasized ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 2), SourcePosition::new(1, 21))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::new(1, 21), SourcePosition::new(1, 23))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("some".to_string()),
                SourceSpan::new(SourcePosition::new(1, 23), SourcePosition::new(1, 27))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 27), SourcePosition::new(1, 29))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text(" way".to_string()),
                SourceSpan::new(SourcePosition::new(1, 29), SourcePosition::new(1, 33))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicEnd,
                SourceSpan::new(SourcePosition::new(1, 33), SourcePosition::new(1, 34))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_mixed_bold_and_italic_emphasis_bold_first() {
        let src = "**This is emphasized *some* way**";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 34)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 3))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("This is emphasized ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 3), SourcePosition::new(1, 22))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::new(1, 22), SourcePosition::new(1, 23))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("some".to_string()),
                SourceSpan::new(SourcePosition::new(1, 23), SourcePosition::new(1, 27))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicEnd,
                SourceSpan::new(SourcePosition::new(1, 27), SourcePosition::new(1, 28))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text(" way".to_string()),
                SourceSpan::new(SourcePosition::new(1, 28), SourcePosition::new(1, 32))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 32), SourcePosition::new(1, 34))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic() {
        let src = "***THIS IS TEXT***";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 19)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 2))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::new(1, 2), SourcePosition::new(1, 4))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("THIS IS TEXT".to_string()),
                SourceSpan::new(SourcePosition::new(1, 4), SourcePosition::new(1, 16))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 16), SourcePosition::new(1, 18))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicEnd,
                SourceSpan::new(SourcePosition::new(1, 18), SourcePosition::new(1, 19))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_2() {
        let src = "***THIS IS TEXT**Hello World*";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 30)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 2))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::new(1, 2), SourcePosition::new(1, 4))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("THIS IS TEXT".to_string()),
                SourceSpan::new(SourcePosition::new(1, 4), SourcePosition::new(1, 16))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 16), SourcePosition::new(1, 18))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Hello World".to_string()),
                SourceSpan::new(SourcePosition::new(1, 18), SourcePosition::new(1, 29))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicEnd,
                SourceSpan::new(SourcePosition::new(1, 29), SourcePosition::new(1, 30))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_3() {
        let src = "*Hello World**THIS IS TEXT***";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 30)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 2))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Hello World".to_string()),
                SourceSpan::new(SourcePosition::new(1, 2), SourcePosition::new(1, 13))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::new(1, 13), SourcePosition::new(1, 15))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("THIS IS TEXT".to_string()),
                SourceSpan::new(SourcePosition::new(1, 15), SourcePosition::new(1, 27))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 27), SourcePosition::new(1, 29))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicEnd,
                SourceSpan::new(SourcePosition::new(1, 29), SourcePosition::new(1, 30))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_4() {
        let src = "**Hello World*THIS IS TEXT***";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 30)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 3))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Hello World".to_string()),
                SourceSpan::new(SourcePosition::new(1, 3), SourcePosition::new(1, 14))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::new(1, 14), SourcePosition::new(1, 15))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("THIS IS TEXT".to_string()),
                SourceSpan::new(SourcePosition::new(1, 15), SourcePosition::new(1, 27))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicEnd,
                SourceSpan::new(SourcePosition::new(1, 27), SourcePosition::new(1, 28))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 28), SourcePosition::new(1, 30))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_5() {
        let src = "***THIS IS TEXT*Hello World**";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 30)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 3))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::new(1, 3), SourcePosition::new(1, 4))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("THIS IS TEXT".to_string()),
                SourceSpan::new(SourcePosition::new(1, 4), SourcePosition::new(1, 16))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicEnd,
                SourceSpan::new(SourcePosition::new(1, 16), SourcePosition::new(1, 17))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Hello World".to_string()),
                SourceSpan::new(SourcePosition::new(1, 17), SourcePosition::new(1, 28))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 28), SourcePosition::new(1, 30))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_6() {
        let src = "Here are *some **stars** for you*";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 34)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Here are ".to_string()),
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 10))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::new(1, 10), SourcePosition::new(1, 11))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("some ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 11), SourcePosition::new(1, 16))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::new(1, 16), SourcePosition::new(1, 18))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("stars".to_string()),
                SourceSpan::new(SourcePosition::new(1, 18), SourcePosition::new(1, 23))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 23), SourcePosition::new(1, 25))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text(" for you".to_string()),
                SourceSpan::new(SourcePosition::new(1, 25), SourcePosition::new(1, 33))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicEnd,
                SourceSpan::new(SourcePosition::new(1, 33), SourcePosition::new(1, 34))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_7() {
        let src = "Here are **some *stars* for you**";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 34)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Here are ".to_string()),
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 10))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::new(1, 10), SourcePosition::new(1, 12))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("some ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 12), SourcePosition::new(1, 17))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::new(1, 17), SourcePosition::new(1, 18))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("stars".to_string()),
                SourceSpan::new(SourcePosition::new(1, 18), SourcePosition::new(1, 23))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicEnd,
                SourceSpan::new(SourcePosition::new(1, 23), SourcePosition::new(1, 24))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text(" for you".to_string()),
                SourceSpan::new(SourcePosition::new(1, 24), SourcePosition::new(1, 32))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 32), SourcePosition::new(1, 34))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_bold_and_italic_escaped() {
        let src = r#"**There is a \* star**"#;

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 22)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 3))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("There is a * star".to_string()),
                SourceSpan::new(SourcePosition::new(1, 3), SourcePosition::new(1, 21))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 21), SourcePosition::new(1, 23))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_code_emphasis_trivial() {
        let src = "`This is emphasized`";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 21)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                CodeStart,
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 2))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("This is emphasized".to_string()),
                SourceSpan::new(SourcePosition::new(1, 2), SourcePosition::new(1, 20))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                CodeEnd,
                SourceSpan::new(SourcePosition::new(1, 20), SourcePosition::new(1, 21))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_code_emphasis_in_context() {
        let src = "Here is some `emphasized` text";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 31)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Here is some ".to_string()),
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 14))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                CodeStart,
                SourceSpan::new(SourcePosition::new(1, 14), SourcePosition::new(1, 15))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("emphasized".to_string()),
                SourceSpan::new(SourcePosition::new(1, 15), SourcePosition::new(1, 25))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                CodeEnd,
                SourceSpan::new(SourcePosition::new(1, 25), SourcePosition::new(1, 26))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text(" text".to_string()),
                SourceSpan::new(SourcePosition::new(1, 26), SourcePosition::new(1, 31))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_code_emphasis_and_ignore_special_chars() {
        let src = r#"In `*this*` case, the \* should be ignored"#;

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 43)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("In ".to_string()),
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 4))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                CodeStart,
                SourceSpan::new(SourcePosition::new(1, 4), SourcePosition::new(1, 5))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("*this*".to_string()),
                SourceSpan::new(SourcePosition::new(1, 5), SourcePosition::new(1, 11))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                CodeEnd,
                SourceSpan::new(SourcePosition::new(1, 11), SourcePosition::new(1, 12))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text(" case, the * should be ignored".to_string()),
                SourceSpan::new(SourcePosition::new(1, 12), SourcePosition::new(1, 43))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_code_emphasis_and_ignore_formatting() {
        let src = "We have some *formatting `*code*`* and in the middle is `code`";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 63)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("We have some ".to_string()),
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 14))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::new(1, 14), SourcePosition::new(1, 15))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("formatting ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 15), SourcePosition::new(1, 26))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                CodeStart,
                SourceSpan::new(SourcePosition::new(1, 26), SourcePosition::new(1, 27))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("*code*".to_string()),
                SourceSpan::new(SourcePosition::new(1, 27), SourcePosition::new(1, 33))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                CodeEnd,
                SourceSpan::new(SourcePosition::new(1, 33), SourcePosition::new(1, 34))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicEnd,
                SourceSpan::new(SourcePosition::new(1, 34), SourcePosition::new(1, 35))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text(" and in the middle is ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 35), SourcePosition::new(1, 57))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                CodeStart,
                SourceSpan::new(SourcePosition::new(1, 57), SourcePosition::new(1, 58))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("code".to_string()),
                SourceSpan::new(SourcePosition::new(1, 58), SourcePosition::new(1, 62))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                CodeEnd,
                SourceSpan::new(SourcePosition::new(1, 62), SourcePosition::new(1, 63))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_link() {
        let src = "Last, but not least, we want some 
links like [here](https://example.com).";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(2, 40)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Last, but not least, we want some links like ".to_string()),
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(2, 12))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Link {
                    label: "here".to_string(),
                    target: "https://example.com".to_string(),
                },
                SourceSpan::new(SourcePosition::new(2, 12), SourcePosition::new(2, 39))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text(".".to_string()),
                SourceSpan::new(SourcePosition::new(2, 39), SourcePosition::new(2, 40))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_image() {
        let src = "Here is an inline image ![alt text](https://example.com/image.png).";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 68)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Here is an inline image ".to_string()),
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 25))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Image {
                    label: "alt text".to_string(),
                    src: "https://example.com/image.png".to_string(),
                },
                SourceSpan::new(SourcePosition::new(1, 25), SourcePosition::new(1, 67))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text(".".to_string()),
                SourceSpan::new(SourcePosition::new(1, 67), SourcePosition::new(1, 68))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_function() {
        let src = "Sometimes we want to use
a function like #Image(
    width: 200px,
    height: 100px,
    src: https://example.com/image.png,
) to do more elaborate things!";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(6, 31)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Sometimes we want to use a function like ".to_string()),
                SourceSpan::new(SourcePosition::zero(), SourcePosition::new(2, 17))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Function {
                    name: "Image".to_string(),
                    parameters: HashMap::from([
                        ("width".to_string(), "200px".to_string()),
                        ("height".to_string(), "100px".to_string()),
                        (
                            "src".to_string(),
                            "https://example.com/image.png".to_string()
                        ),
                    ])
                },
                SourceSpan::new(SourcePosition::new(2, 17), SourcePosition::new(6, 2))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text(" to do more elaborate things!".to_string()),
                SourceSpan::new(SourcePosition::new(6, 2), SourcePosition::new(6, 31))
            )
        );
        assert!(tokenizer.next().is_none());
    }
}
