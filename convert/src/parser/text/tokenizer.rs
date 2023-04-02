use crate::parser::text::token::Token;
use crate::util::{SourcePosition, SourceSpan};

pub(crate) struct Tokenizer {
    src: String,
    offset: usize,
    offset_source_position: SourcePosition,
    last_char_source_position: SourcePosition,
    span: SourceSpan,
}

impl Tokenizer {
    pub(crate) fn new(src: String, span: SourceSpan) -> Self {
        Self {
            src,
            offset: 0,
            offset_source_position: span.start.clone(),
            last_char_source_position: span.start.clone(),
            span,
        }
    }
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::text::token::TokenKind::{
        BoldEnd, BoldStart, CodeEnd, CodeStart, Image, ItalicEnd, ItalicStart, Link, Text,
    };

    use super::*;

    #[test]
    fn tokenize_trivial() {
        let src = "This is a simple paragraph.";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("This is a simple paragraph.".to_string()),
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_trivial_with_source_positions() {
        let src = "This is a simple paragraph
that spans over multiple
lines.";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(3, 7)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("This is a simple paragraph that spans over multiple lines.".to_string()),
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(3, 7)),
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_italic_emphasis() {
        let src = "*This is emphasized*";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 2))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 3))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 2))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 3))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 2))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 2))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                ItalicStart,
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 2))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 3))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 3))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Here are ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 10))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Here are ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 10))
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
        let src = "**There is a \\* star**";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldStart,
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 3))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("There is a * star".to_string()),
                SourceSpan::new(SourcePosition::new(1, 3), SourcePosition::new(1, 20))
            )
        );
        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                BoldEnd,
                SourceSpan::new(SourcePosition::new(1, 20), SourcePosition::new(1, 22))
            )
        );
        assert!(tokenizer.next().is_none());
    }

    #[test]
    fn tokenize_code_emphasis_trivial() {
        let src = "`This is emphasized`";

        let mut tokenizer = Tokenizer::new(
            src.to_string(),
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                CodeStart,
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 2))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Here is some ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 14))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("In ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 4))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, src.len())),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("We have some ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 14))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(2, 40)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Last, but not least, we want some links like ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(2, 12))
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
            SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(2, 40)),
        );

        assert_eq!(
            tokenizer.next().unwrap(),
            Token::new(
                Text("Here is an inline image ".to_string()),
                SourceSpan::new(SourcePosition::new(1, 1), SourcePosition::new(1, 25))
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
}
