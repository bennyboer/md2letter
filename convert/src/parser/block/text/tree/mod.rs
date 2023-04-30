use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use crate::util::{IdGenerator, SourceSpan};

pub(crate) use self::node::{TextNode, TextNodeId, TextNodeKind};

mod node;

#[derive(Debug)]
pub(crate) struct TextTree {
    nodes: HashMap<TextNodeId, TextNode>,
    root: TextNodeId,
    node_id_generator: IdGenerator,
}

struct NodeOnLevel {
    node_id: TextNodeId,
    level: usize,
}

impl TextTree {
    pub(crate) fn new(span: SourceSpan) -> Self {
        let mut node_id_generator = IdGenerator::new();
        let mut nodes = HashMap::new();

        let root_id = node_id_generator.next();
        let root_node = TextNode::new(root_id, TextNodeKind::Root, span);
        nodes.insert(root_id, root_node);

        Self {
            nodes,
            root: root_id,
            node_id_generator,
        }
    }

    pub(crate) fn root(&self) -> &TextNode {
        self.nodes.get(&self.root).unwrap()
    }

    pub(crate) fn get_node(&self, id: TextNodeId) -> &TextNode {
        self.nodes.get(&id).unwrap()
    }

    pub(crate) fn register_node(
        &mut self,
        parent_id: TextNodeId,
        kind: TextNodeKind,
        span: SourceSpan,
    ) -> TextNodeId {
        let id = self.node_id_generator.next();

        let node = TextNode::new(id, kind, span);
        self.nodes.insert(id, node);

        let parent_node = self.nodes.get_mut(&parent_id).unwrap();
        parent_node.register_child(id);

        id
    }

    fn visit(&self, node: &TextNode, nodes: &mut Vec<NodeOnLevel>, level: usize) {
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

impl Display for TextTree {
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
