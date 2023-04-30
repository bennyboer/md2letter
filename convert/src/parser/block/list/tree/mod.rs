use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub(crate) use crate::parser::block::list::tree::node::{
    ListNode, ListNodeId, ListNodeKind, ListNodeStyle,
};
use crate::util::IdGenerator;

mod node;

#[derive(Debug)]
pub(crate) struct ListTree {
    items: HashMap<ListNodeId, ListNode>,
    root: ListNodeId,
    node_id_generator: IdGenerator,
}

struct ListNodeOnLevel {
    node_id: ListNodeId,
    level: usize,
}

impl ListTree {
    pub fn new() -> Self {
        let mut node_id_generator = IdGenerator::new();
        let mut items = HashMap::new();

        let root_id = node_id_generator.next();
        let root_node = ListNode::new(root_id, ListNodeKind::Parent, ListNodeStyle::Unordered);
        items.insert(root_id, root_node);

        Self {
            items,
            root: root_id,
            node_id_generator,
        }
    }

    pub(crate) fn root(&self) -> &ListNode {
        self.items.get(&self.root).unwrap()
    }

    pub(crate) fn get_node(&self, id: ListNodeId) -> &ListNode {
        self.items.get(&id).unwrap()
    }

    pub(crate) fn register_node(
        &mut self,
        parent_id: ListNodeId,
        kind: ListNodeKind,
        style: ListNodeStyle,
    ) -> ListNodeId {
        let id = self.node_id_generator.next();

        let node = ListNode::new(id, kind, style);
        self.items.insert(id, node);

        let parent_node = self.items.get_mut(&parent_id).unwrap();
        parent_node.register_child(id);

        id
    }

    fn visit(&self, node: &ListNode, nodes: &mut Vec<ListNodeOnLevel>, level: usize) {
        nodes.push(ListNodeOnLevel {
            node_id: node.id(),
            level,
        });

        for child_id in node.children() {
            if let Some(child) = self.items.get(child_id) {
                self.visit(child, nodes, level + 1);
            }
        }
    }
}

impl Display for ListTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut nodes = Vec::new();
        self.visit(self.root(), &mut nodes, 0);

        for node in nodes {
            let level = node.level;

            if let Some(node) = self.items.get(&node.node_id) {
                for _ in 0..level {
                    write!(f, "  ")?;
                }

                if let ListNodeKind::Leaf { .. } = node.kind() {
                    writeln!(f, "- {} {}", node.style(), node.kind().to_string(level))?;
                } else {
                    writeln!(f, "- {}", node.kind().to_string(level))?;
                }
            }
        }

        Ok(())
    }
}
