//! Categorize Markdown blocks (Text, Heading, List, etc.).
//! ---
//! Alternatively we could just try to parse using each parser and see if they'd fail.
//! That would not be very good performance wise, as we'd need to question each parser.
//! The categorization thus is just a performance optimization.
//! If the categorization is wrong - meaning that the designated parser is not able to figure out what the content means - the text parser is used as a fallback.

use crate::categorizer::block::BlockKind::{
    Code, Function, Heading, HorizontalRule, Image, List, Quote, Table, Text,
};
pub(crate) use crate::categorizer::block::{BlockKind, CategorizedBlock};
use crate::splitter::SplitterBlock;

pub(crate) struct BlockCategorizer;

mod block;

impl BlockCategorizer {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn categorize(&self, block: SplitterBlock) -> CategorizedBlock {
        let source_span = block.span().clone();
        let src = block.into_src();

        let first_char = src.chars().next().unwrap();
        let kind = match first_char {
            '#' => {
                if self.is_heading(&src) {
                    Heading
                } else if self.is_function_block(&src) {
                    Function
                } else {
                    Text
                }
            }
            '!' => {
                if self.is_image(&src) {
                    Image
                } else {
                    Text
                }
            }
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                if self.is_ordered_list(&src) {
                    List
                } else {
                    Text
                }
            }
            '-' => {
                if self.is_horizontal_rule(&src, first_char) {
                    HorizontalRule
                } else if self.is_unordered_list(&src, first_char) {
                    List
                } else {
                    Text
                }
            }
            '+' => {
                if self.is_horizontal_rule(&src, first_char) {
                    HorizontalRule
                } else if self.is_unordered_list(&src, first_char) {
                    List
                } else {
                    Text
                }
            }
            '*' => {
                if self.is_horizontal_rule(&src, first_char) {
                    HorizontalRule
                } else if self.is_unordered_list(&src, first_char) {
                    List
                } else {
                    Text
                }
            }
            '_' => {
                if self.is_horizontal_rule(&src, first_char) {
                    HorizontalRule
                } else {
                    Text
                }
            }
            '>' => Quote,
            '|' => Table,
            '`' => {
                if self.is_code_block(&src) {
                    Code
                } else {
                    Text
                }
            }
            _ => Text,
        };

        CategorizedBlock::new(kind, src, source_span)
    }

    fn is_code_block(&self, src: &str) -> bool {
        let mut counter = 0;

        for c in src.chars() {
            if c == '`' {
                counter += 1;
            } else {
                break;
            }
        }

        counter >= 3
    }

    fn is_ordered_list(&self, src: &str) -> bool {
        let next_char_is_period = src
            .chars()
            .skip(1)
            .next()
            .map(|c| c == '.')
            .unwrap_or(false);
        let next_char_is_space = next_char_is_period
            && src
                .chars()
                .skip(2)
                .next()
                .map(|c| c == ' ')
                .unwrap_or(false);

        next_char_is_period && next_char_is_space
    }

    fn is_unordered_list(&self, src: &str, _char: char) -> bool {
        let next_char_is_space = src
            .chars()
            .skip(1)
            .next()
            .map(|c| c == ' ')
            .unwrap_or(false);

        next_char_is_space
    }

    fn is_horizontal_rule(&self, src: &str, char: char) -> bool {
        let mut counter = 0;

        for c in src.chars() {
            if c == char {
                counter += 1;
            } else {
                break;
            }
        }

        let is_possible_horizontal_rule = counter >= 3;

        // Check if there is more content to the block
        if is_possible_horizontal_rule {
            for c in src.chars().skip(counter) {
                match c {
                    ' ' | '\t' => {}
                    _ => return false,
                }
            }
        }

        is_possible_horizontal_rule
    }

    fn is_image(&self, src: &str) -> bool {
        if let Some(c) = src.chars().skip(1).next() {
            if c != '[' {
                return false;
            }
        } else {
            return false;
        }

        let mut counter = 2;

        // Find closing square bracket
        for c in src.chars().skip(counter) {
            counter += 1;

            if c == ']' {
                break;
            }
        }

        // Find opening parenthesis
        for c in src.chars().skip(counter) {
            counter += 1;

            if c == '(' {
                break;
            }
        }

        // Find closing parenthesis
        let mut may_be_image_block = false;
        for c in src.chars().skip(counter) {
            counter += 1;

            if c == ')' {
                may_be_image_block = true;
                break;
            }
        }

        // Check if there is more content in the block
        for c in src.chars().skip(counter) {
            match c {
                ' ' | '\t' | '\n' => {}
                _ => return false,
            }
        }

        return may_be_image_block;
    }

    fn is_function_block(&self, src: &str) -> bool {
        let mut counter = 1;

        // Find function name
        let mut has_name = false;
        let mut anticipate_params = false;
        for c in src.chars().skip(counter) {
            counter += 1;

            match c {
                '(' => {
                    if has_name {
                        anticipate_params = true;
                        break;
                    }

                    return false;
                }
                '\t' | '#' => return false,
                ' ' => {
                    if has_name {
                        // May be function without parameters like '#break'
                        break;
                    } else {
                        return false;
                    }
                }
                _ => {
                    has_name = true;
                }
            }
        }

        // Find function parameters
        let mut params_are_valid = !anticipate_params;
        if anticipate_params {
            for c in src.chars().skip(counter) {
                counter += 1;

                if c == ')' {
                    params_are_valid = true;
                    break;
                }
            }
        }

        if !params_are_valid {
            return false;
        }

        // Check if there is more content to the block than just the function
        for c in src.chars().skip(counter) {
            match c {
                ' ' | '\t' | '\n' => {}
                _ => return false,
            }
        }

        return true;
    }

    fn is_heading(&self, src: &str) -> bool {
        let mut chars = src.chars();

        chars.next(); // Skip the first char

        for c in chars {
            match c {
                ' ' => return true,
                '#' => continue,
                _ => return false,
            }
        }

        return false;
    }
}

#[cfg(test)]
mod tests {
    use crate::categorizer::block::BlockKind::{Code, HorizontalRule, List, Quote, Table};
    use crate::util::SourcePosition;
    use crate::util::SourceSpan;

    use super::*;

    #[test]
    fn categorize_text() {
        let text_block = SplitterBlock::new(
            "Hello World [click here](https://example.com) - it's **cool**!".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 63)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(text_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(
            categorized_block.src(),
            "Hello World [click here](https://example.com) - it's **cool**!"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 63))
        );
    }

    #[test]
    fn categorize_code() {
        let code_block = SplitterBlock::new(
            "```js
console.log('test');
```"
            .to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 4)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(code_block);

        assert_eq!(categorized_block.kind(), &Code);
        assert_eq!(
            categorized_block.src(),
            "```js
console.log('test');
```"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 4))
        );
    }

    #[test]
    fn categorize_faulty_code_as_text() {
        let code_block = SplitterBlock::new(
            "` ``js
console.log('test');
```"
            .to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 4)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(code_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(
            categorized_block.src(),
            "` ``js
console.log('test');
```"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 4))
        );
    }

    #[test]
    fn categorize_quote() {
        let quote_block = SplitterBlock::new(
            "> Hello World".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 14)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(quote_block);

        assert_eq!(categorized_block.kind(), &Quote);
        assert_eq!(categorized_block.src(), "> Hello World");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 14))
        );
    }

    #[test]
    fn categorize_table() {
        let table_block = SplitterBlock::new(
            "| First Header  | Second Header |
| ------------- | ------------- |
| Content Cell  | Content Cell  |"
                .to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 34)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(table_block);

        assert_eq!(categorized_block.kind(), &Table);
        assert_eq!(
            categorized_block.src(),
            "| First Header  | Second Header |
| ------------- | ------------- |
| Content Cell  | Content Cell  |"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 34))
        );
    }

    #[test]
    fn categorize_ordered_list() {
        let list_block = SplitterBlock::new(
            "1. First item
2. Second item
3. Third item"
                .to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 14)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(list_block);

        assert_eq!(categorized_block.kind(), &List);
        assert_eq!(
            categorized_block.src(),
            "1. First item
2. Second item
3. Third item"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 14))
        );
    }

    #[test]
    fn categorize_unordered_list_with_minus_char() {
        let list_block = SplitterBlock::new(
            "- First item
- Second item
- Third item"
                .to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 13)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(list_block);

        assert_eq!(categorized_block.kind(), &List);
        assert_eq!(
            categorized_block.src(),
            "- First item
- Second item
- Third item"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 13))
        );
    }

    #[test]
    fn categorize_unordered_list_with_plus_char() {
        let list_block = SplitterBlock::new(
            "+ First item
+ Second item
+ Third item"
                .to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 13)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(list_block);

        assert_eq!(categorized_block.kind(), &List);
        assert_eq!(
            categorized_block.src(),
            "+ First item
+ Second item
+ Third item"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 13))
        );
    }

    #[test]
    fn categorize_unordered_list_with_star_char() {
        let list_block = SplitterBlock::new(
            "* First item
* Second item
* Third item"
                .to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 13)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(list_block);

        assert_eq!(categorized_block.kind(), &List);
        assert_eq!(
            categorized_block.src(),
            "* First item
* Second item
* Third item"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 13))
        );
    }

    #[test]
    fn categorize_nested_list() {
        let list_block = SplitterBlock::new(
            "- First item
    - Second item
    - Third item"
                .to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 17)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(list_block);

        assert_eq!(categorized_block.kind(), &List);
        assert_eq!(
            categorized_block.src(),
            "- First item
    - Second item
    - Third item"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 17))
        );
    }

    #[test]
    fn categorize_list_with_first_item_indented_as_text() {
        let list_block = SplitterBlock::new(
            "   - First item
    - Second item
    - Third item"
                .to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 17)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(list_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(
            categorized_block.src(),
            "   - First item
    - Second item
    - Third item"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(3, 17))
        );
    }

    #[test]
    fn categorize_horizontal_rule_with_minus_char() {
        let horizontal_rule_block = SplitterBlock::new(
            "---".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 4)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(horizontal_rule_block);

        assert_eq!(categorized_block.kind(), &HorizontalRule);
        assert_eq!(categorized_block.src(), "---");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 4))
        );
    }

    #[test]
    fn categorize_horizontal_rule_with_star_char() {
        let horizontal_rule_block = SplitterBlock::new(
            "***".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 4)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(horizontal_rule_block);

        assert_eq!(categorized_block.kind(), &HorizontalRule);
        assert_eq!(categorized_block.src(), "***");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 4))
        );
    }

    #[test]
    fn categorize_faulty_horizontal_rule_with_stars_as_text() {
        let horizontal_rule_block = SplitterBlock::new(
            "***Some text***".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 16)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(horizontal_rule_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(categorized_block.src(), "***Some text***");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 16))
        );
    }

    #[test]
    fn categorize_horizontal_rule_with_plus_char() {
        let horizontal_rule_block = SplitterBlock::new(
            "+++".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 4)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(horizontal_rule_block);

        assert_eq!(categorized_block.kind(), &HorizontalRule);
        assert_eq!(categorized_block.src(), "+++");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 4))
        );
    }

    #[test]
    fn categorize_horizontal_rule_with_underscore_char() {
        let horizontal_rule_block = SplitterBlock::new(
            "___".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 4)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(horizontal_rule_block);

        assert_eq!(categorized_block.kind(), &HorizontalRule);
        assert_eq!(categorized_block.src(), "___");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 4))
        );
    }

    #[test]
    fn categorize_horizontal_rule_with_a_lot_of_chars() {
        let horizontal_rule_block = SplitterBlock::new(
            "--------------------------".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 27)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(horizontal_rule_block);

        assert_eq!(categorized_block.kind(), &HorizontalRule);
        assert_eq!(categorized_block.src(), "--------------------------");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 27))
        );
    }

    #[test]
    fn categorize_horizontal_rule_with_less_than_three_chars_as_text() {
        let horizontal_rule_block = SplitterBlock::new(
            "--".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 3)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(horizontal_rule_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(categorized_block.src(), "--");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 3))
        );
    }

    #[test]
    fn categorize_heading() {
        let heading_block = SplitterBlock::new(
            "# This is a heading".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 20)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(heading_block);

        assert_eq!(categorized_block.kind(), &Heading);
        assert_eq!(categorized_block.src(), "# This is a heading");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 20))
        );
    }

    #[test]
    fn categorize_image() {
        let image_block = SplitterBlock::new(
            "![This is an image](https://example.com/image.png)".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 51)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(image_block);

        assert_eq!(categorized_block.kind(), &Image);
        assert_eq!(
            categorized_block.src(),
            "![This is an image](https://example.com/image.png)"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 51))
        );
    }

    #[test]
    fn categorize_image_with_empty_tag() {
        let image_block = SplitterBlock::new(
            "![](https://example.com/image.png)".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 35)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(image_block);

        assert_eq!(categorized_block.kind(), &Image);
        assert_eq!(
            categorized_block.src(),
            "![](https://example.com/image.png)"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 35))
        );
    }

    #[test]
    fn categorize_faulty_image_as_text() {
        let image_block = SplitterBlock::new(
            "!(https://example.com/image.png)".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 33)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(image_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(categorized_block.src(), "!(https://example.com/image.png)");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 33))
        );
    }

    #[test]
    fn categorize_faulty_image_as_text_2() {
        let image_block = SplitterBlock::new(
            "!".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 2)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(image_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(categorized_block.src(), "!");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 2))
        );
    }

    #[test]
    fn categorize_faulty_image_as_text_3() {
        let image_block = SplitterBlock::new(
            "![tag]".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 7)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(image_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(categorized_block.src(), "![tag]");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 7))
        );
    }

    #[test]
    fn categorize_image_followed_by_text_as_text() {
        let image_block = SplitterBlock::new(
            "![tag](of_image_src) hello world".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 33)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(image_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(categorized_block.src(), "![tag](of_image_src) hello world");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 33))
        );
    }

    #[test]
    fn categorize_text_starting_with_function() {
        let text_block = SplitterBlock::new(
            "#fn(test) Hello World".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 22)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(text_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(categorized_block.src(), "#fn(test) Hello World");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 22))
        );
    }

    #[test]
    fn categorize_function_without_name_as_text() {
        let text_block = SplitterBlock::new(
            "#(test)".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 8)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(text_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(categorized_block.src(), "#(test)");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 8))
        );
    }

    #[test]
    fn categorize_function_without_params_followed_by_text_as_text() {
        let text_block = SplitterBlock::new(
            "#break and some text".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 21)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(text_block);

        assert_eq!(categorized_block.kind(), &Text);
        assert_eq!(categorized_block.src(), "#break and some text");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 21))
        );
    }

    #[test]
    fn categorize_function() {
        let function_block = SplitterBlock::new(
            "\
#image(
    width: 100px, 
    height: 100px, 
    src: \"test.jpg\"
)\
"
            .to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(5, 1)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(function_block);

        assert_eq!(categorized_block.kind(), &Function);
        assert_eq!(
            categorized_block.src(),
            "\
#image(
    width: 100px, 
    height: 100px, 
    src: \"test.jpg\"
)\
"
        );
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(5, 1))
        );
    }

    #[test]
    fn categorize_function_without_params() {
        let function_block = SplitterBlock::new(
            "#break".to_string(),
            SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 7)),
        );

        let categorizer = BlockCategorizer::new();

        let categorized_block = categorizer.categorize(function_block);

        assert_eq!(categorized_block.kind(), &Function);
        assert_eq!(categorized_block.src(), "#break");
        assert_eq!(
            categorized_block.span(),
            &SourceSpan::new(SourcePosition::zero(), SourcePosition::new(1, 7))
        );
    }
}
