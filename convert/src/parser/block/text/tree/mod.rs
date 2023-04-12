use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::util::{IdGenerator, SourceSpan};

pub(crate) use self::node::{Node, NodeId, NodeKind};

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

    pub(crate) fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
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

    fn visit(&self, node: &Node, nodes: &mut Vec<NodeOnLevel>, level: usize) {
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

impl Display for Tree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut nodes = Vec::new();
        self.visit(self.root(), &mut nodes, 0);

        for node in nodes {
            let level = node.level;

            if let Some(node) = self.nodes.get(&node.node_id) {
                for _ in 0..level {
                    write!(f, "  ")?;
                }

                writeln!(f, "- {}", node.kind())?;
            }
        }

        Ok(())
    }
}

struct NodeOnLevel {
    node_id: NodeId,
    level: usize,
}
