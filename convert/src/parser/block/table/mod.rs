use self::cell::TableCell;

mod cell;

pub(crate) type TableRow = Vec<TableCell>;

pub(crate) struct TableBlock {
    header_row: TableRow,
    rows: Vec<TableRow>,
}
