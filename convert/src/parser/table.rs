use crate::parser::block::table::{TableBlock, TableCell, TableRow};
use crate::parser::block::ParsedBlockKind;
use crate::parser::text::TextParser;
use crate::parser::{ParseResult, ParsedBlock};
use crate::util::{SourcePosition, SourceSpan};

pub(crate) struct TableParser {
    src: String,
    span: SourceSpan,
    header_row: TableRow,
    rows: Vec<TableRow>,
}

#[derive(Copy, Clone)]
enum RowKind {
    Header,
    HeaderSeparator,
    Body,
}

impl TableParser {
    pub fn new(src: String, span: SourceSpan) -> Self {
        Self {
            src,
            span,
            header_row: Vec::new(),
            rows: Vec::new(),
        }
    }

    pub fn parse(mut self) -> ParseResult<ParsedBlock> {
        let src = self.src.clone();

        for (row_index, line) in src.lines().enumerate() {
            let line_number = self.span.start.line + row_index;

            let mut started_row = false;
            let mut offset = 1;
            let mut cell_value_buffer = String::new();

            for c in line.chars() {
                match c {
                    '|' => {
                        if started_row {
                            self.consume_buffer_and_register_cell(
                                &mut cell_value_buffer,
                                line_number,
                                offset,
                                row_index,
                            )?;
                        } else {
                            started_row = true;
                        }
                    }
                    '\n' => {
                        started_row = false;
                    }
                    _ => {
                        cell_value_buffer.push(c);
                    }
                }

                offset += 1;
            }
        }

        Ok(ParsedBlock::new(
            ParsedBlockKind::Table(TableBlock::new(self.header_row, self.rows)),
            self.span,
        ))
    }

    fn consume_buffer_and_register_cell(
        &mut self,
        cell_value_buffer: &mut String,
        line_number: usize,
        offset: usize,
        row_index: usize,
    ) -> ParseResult<()> {
        let row_kind = RowKind::for_line_index(row_index);
        if let RowKind::HeaderSeparator = row_kind {
            cell_value_buffer.clear();
            return Ok(());
        }

        let cell = self.create_cell(&cell_value_buffer, line_number, offset)?;

        match row_kind {
            RowKind::Header => self.header_row.push(cell),
            RowKind::Body => {
                let internal_row_index = row_index - 2;
                if self.rows.len() <= internal_row_index {
                    self.rows.push(Vec::new());
                }

                self.rows.last_mut().unwrap().push(cell)
            }
            _ => unreachable!(),
        }

        cell_value_buffer.clear();
        Ok(())
    }

    fn create_cell(
        &self,
        value: &str,
        line_number: usize,
        offset: usize,
    ) -> ParseResult<TableCell> {
        let trimmed_value = value.trim();

        let span = SourceSpan::new(
            SourcePosition::new(line_number, offset - value.len()),
            SourcePosition::new(line_number, offset - (value.len() - trimmed_value.len())),
        );
        let text_parser = TextParser::new(trimmed_value.to_string(), span);
        let parsed_block = text_parser.parse()?;
        let text_block = if let ParsedBlockKind::Text(text_block) = parsed_block.into_kind() {
            text_block
        } else {
            unreachable!();
        };
        let text_tree = text_block.into_tree();

        Ok(TableCell::new(text_tree))
    }
}

impl RowKind {
    fn for_line_index(line_index: usize) -> Self {
        match line_index {
            0 => RowKind::Header,
            1 => RowKind::HeaderSeparator,
            _ => RowKind::Body,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::SourcePosition;

    use super::*;

    #[test]
    fn should_parse_simple_table() {
        let src = "| Column A | Column B |
| -------- | -------- |
| 1        | 2        |
| 3        | 4        |";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(4, 24));
        let parser = TableParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_table());

        let table_block = if let ParsedBlockKind::Table(table_block) = parsed_block.into_kind() {
            table_block
        } else {
            panic!("Expected table block");
        };

        let header_row = table_block.header_row();
        assert_eq!(header_row.len(), 2);

        assert_eq!(
            header_row.get(0).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](Column A)
"
        );
        assert_eq!(
            header_row.get(1).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](Column B)
"
        );

        assert_eq!(table_block.row_count(), 2);

        let row_1 = table_block.get_row(0).unwrap();
        assert_eq!(row_1.len(), 2);

        assert_eq!(
            row_1.get(0).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](1)
"
        );
        assert_eq!(
            row_1.get(1).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](2)
"
        );

        let row_2 = table_block.get_row(1).unwrap();
        assert_eq!(row_2.len(), 2);

        assert_eq!(
            row_2.get(0).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](3)
"
        );
        assert_eq!(
            row_2.get(1).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](4)
"
        );
    }

    #[test]
    fn should_parse_table_with_formatting() {
        let src = "| Column *A* | Column *B* |
| --- | --- |
| 1        | Some **bold** text |
| 3        | 4        |";
        let span = SourceSpan::new(SourcePosition::zero(), SourcePosition::new(4, 24));
        let parser = TableParser::new(src.to_string(), span);

        let result = parser.parse();
        assert!(result.is_ok());

        let parsed_block = result.unwrap();
        assert!(parsed_block.is_table());

        let table_block = if let ParsedBlockKind::Table(table_block) = parsed_block.into_kind() {
            table_block
        } else {
            panic!("Expected table block");
        };

        let header_row = table_block.header_row();
        assert_eq!(header_row.len(), 2);

        assert_eq!(
            header_row.get(0).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](Column )
  - [Italic]
    - [Text](A)
"
        );
        assert_eq!(
            header_row.get(1).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](Column )
  - [Italic]
    - [Text](B)
"
        );

        assert_eq!(table_block.row_count(), 2);

        let row_1 = table_block.get_row(0).unwrap();
        assert_eq!(row_1.len(), 2);

        assert_eq!(
            row_1.get(0).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](1)
"
        );
        assert_eq!(
            row_1.get(1).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](Some )
  - [Bold]
    - [Text](bold)
  - [Text]( text)
"
        );

        let row_2 = table_block.get_row(1).unwrap();
        assert_eq!(row_2.len(), 2);

        assert_eq!(
            row_2.get(0).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](3)
"
        );
        assert_eq!(
            row_2.get(1).unwrap().text_tree().to_string(),
            "- [Root]
  - [Text](4)
"
        );
    }
}
