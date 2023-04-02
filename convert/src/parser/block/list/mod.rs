use self::tree::ListTree;

mod tree;

#[derive(Debug)]
pub(crate) struct ListBlock {
    is_ordered: bool,
    tree: ListTree,
}
