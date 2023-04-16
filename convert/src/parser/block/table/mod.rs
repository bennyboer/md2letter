pub(crate) use self::cell::TableCell;

mod cell;

pub(crate) type TableRow = Vec<TableCell>;

#[derive(Debug)]
pub(crate) struct TableBlock {
    header_row: TableRow,
    rows: Vec<TableRow>,
}

impl TableBlock {
    pub fn new(header_row: TableRow, rows: Vec<TableRow>) -> Self {
        Self { header_row, rows }
    }

    pub fn header_row(&self) -> &TableRow {
        &self.header_row
    }

    pub fn get_row(&self, index: usize) -> Option<&TableRow> {
        self.rows.get(index)
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}
