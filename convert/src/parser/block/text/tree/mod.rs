use std::collections::HashMap;

use crate::parser::block::text::tree::node::NodeKind;
use crate::util::{IdGenerator, SourceSpan};

use self::node::{Node, NodeId};

mod node;

#[derive(Debug)]
pub(crate) struct Tree {
    nodes: HashMap<NodeId, Node>,
    root: NodeId,
    node_id_generator: IdGenerator,
}

impl Tree {
    pub(crate) fn new(span: SourceSpan) -> Self {
        let mut node_id_generator = IdGenerator::new();
        let mut nodes = HashMap::new();

        let root_id = node_id_generator.next();
        let root_node = Node::new(root_id, NodeKind::Root, span);
        nodes.insert(root_id, root_node);

        Self {
            nodes,
            root: root_id,
            node_id_generator,
        }
    }

    pub(crate) fn root(&self) -> &Node {
        self.nodes.get(&self.root).unwrap()
    }

    pub(crate) fn register_node(
        &mut self,
        parent_id: NodeId,
        kind: NodeKind,
        span: SourceSpan,
    ) -> NodeId {
        let id = self.node_id_generator.next();

        let node = Node::new(id, kind, span);
        self.nodes.insert(id, node);

        let parent_node = self.nodes.get_mut(&parent_id).unwrap();
        parent_node.register_child(id);

        id
    }
}
