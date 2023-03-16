use TokenKind::Emphasis;

use crate::source_position::SourcePosition;
use crate::source_span::SourceSpan;
use crate::tokenizer::state::block_separator::BlockSeparatorState;
use crate::tokenizer::state::heading_level::HeadingLevelState;
use crate::tokenizer::state::horizontal_rule::HorizontalRuleState;
use crate::tokenizer::state::result::{StateProcessResult, StateStackOperation};
use crate::tokenizer::state::text::TextState;
use crate::tokenizer::state::State;
use crate::tokenizer::token::EmphasisType::{BoldOrItalic, Code};
use crate::tokenizer::token::{EmphasisType, Token, TokenKind};
use crate::tokenizer::Tokenizer;

pub(crate) struct StartState;

impl StartState {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl State for StartState {
    fn process(&mut self, c: char, tokenizer: &mut Tokenizer) -> StateProcessResult {
        match c {
            _ => {
                let source_position = tokenizer.source_position();

                let next_state: Box<dyn State> = match c {
                    '\n' => Box::new(BlockSeparatorState::new(source_position)),
                    '#' => Box::new(HeadingLevelState::new(source_position)),
                    '*' => return create_emphasis_token(BoldOrItalic, &source_position),
                    '`' => return create_emphasis_token(Code, &source_position),
                    '-' | '+' | '_' => Box::new(HorizontalRuleState::new(source_position, c)),
                    // TODO List item
                    // TODO Block quote
                    // TODO Code block
                    // TODO Table
                    // TODO Horizontal rule
                    _ => {
                        tokenizer.mark_char_as_unconsumed(c);
                        Box::new(TextState::new(source_position))
                    }
                };

                StateProcessResult::new()
                    .with_state_stack_operation(StateStackOperation::Push(next_state))
            }
        }
    }

    fn end_of_input(&mut self, _tokenizer: &mut Tokenizer) -> StateProcessResult {
        StateProcessResult::new()
    }

    fn name(&self) -> &str {
        "StartState"
    }
}

fn create_emphasis_token(
    emphasisType: EmphasisType,
    source_position: &SourcePosition,
) -> StateProcessResult {
    let source_span = SourceSpan {
        start: SourcePosition {
            line: source_position.line,
            column: source_position.column - 1,
        },
        end: source_position.clone(),
    };
    let token = Token::new(Emphasis(emphasisType), source_span);

    return StateProcessResult::new().with_token(token);
}
