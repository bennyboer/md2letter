use std::collections::HashMap;

use self::node::{Node, NodeId};

mod node;

pub(crate) struct Tree {
    nodes: HashMap<NodeId, Node>,
    root: NodeId,
}
