use self::tree::ListTree;

mod tree;

pub(crate) struct ListBlock {
    is_ordered: bool,
    tree: ListTree,
}
