use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::util::IdGenerator;

pub(crate) use self::node::{QuoteNode, QuoteNodeId, QuoteNodeKind};

mod node;

#[derive(Debug)]
pub(crate) struct QuoteTree {
    nodes: HashMap<QuoteNodeId, QuoteNode>,
    root: QuoteNodeId,
    node_id_generator: IdGenerator,
}

struct NodeOnLevel {
    node_id: QuoteNodeId,
    level: usize,
}

impl QuoteTree {
    pub fn new() -> Self {
        let mut id_generator = IdGenerator::new();
        let root_id = id_generator.next();
        let root_node = QuoteNode::new(root_id, QuoteNodeKind::Parent);

        let mut nodes = HashMap::new();
        nodes.insert(root_id, root_node);

        Self {
            nodes,
            root: root_id,
            node_id_generator: id_generator,
        }
    }

    pub fn root(&self) -> &QuoteNode {
        self.nodes.get(&self.root).unwrap()
    }

    pub(crate) fn get_node(&self, id: QuoteNodeId) -> &QuoteNode {
        self.nodes.get(&id).unwrap()
    }

    pub fn register_node(&mut self, parent: QuoteNodeId, kind: QuoteNodeKind) -> QuoteNodeId {
        let node_id = self.node_id_generator.next();
        let node = QuoteNode::new(node_id, kind);
        self.nodes.insert(node_id, node);
        self.nodes.get_mut(&parent).unwrap().register_child(node_id);

        node_id
    }

    fn visit(&self, node: &QuoteNode, nodes: &mut Vec<NodeOnLevel>, level: usize) {
        nodes.push(NodeOnLevel {
            node_id: node.id(),
            level,
        });

        for child_id in node.children() {
            if let Some(child) = self.nodes.get(child_id) {
                self.visit(child, nodes, level + 1);
            }
        }
    }
}

impl Display for QuoteTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut nodes = Vec::new();
        self.visit(self.root(), &mut nodes, 0);

        for node in nodes {
            let level = node.level;

            if let Some(node) = self.nodes.get(&node.node_id) {
                for _ in 0..level {
                    write!(f, "  ")?;
                }

                writeln!(f, "- {}", node.kind().to_string(level))?;
            }
        }

        Ok(())
    }
}
