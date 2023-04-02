use std::collections::HashMap;

use self::node::{ListNode, ListNodeId};

mod node;

pub(crate) struct ListTree {
    items: HashMap<ListNodeId, ListNode>,
    root: ListNodeId,
}
