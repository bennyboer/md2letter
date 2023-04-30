use std::collections::HashMap;

pub(crate) use crate::transformer::tree::node::{
    LetterScriptNode, LetterScriptNodeId, LetterScriptNodeKind,
};
use crate::util::{IdGenerator, SourcePosition, SourceSpan};

mod node;

pub(crate) struct LetterScriptTree {
    node_lookup: HashMap<LetterScriptNodeId, LetterScriptNode>,
    root_id: LetterScriptNodeId,
    node_id_generator: IdGenerator,
}

impl LetterScriptTree {
    pub fn new() -> Self {
        let mut node_id_generator = IdGenerator::new();
        let root_id = node_id_generator.next();
        let mut node_lookup = HashMap::new();

        let root = LetterScriptNode::new(
            root_id,
            LetterScriptNodeKind::Root,
            SourceSpan::new(SourcePosition::zero(), SourcePosition::zero()),
        );
        node_lookup.insert(root_id, root);

        Self {
            node_lookup,
            root_id,
            node_id_generator,
        }
    }

    pub(crate) fn root_id(&self) -> LetterScriptNodeId {
        self.root_id
    }

    pub(crate) fn get_node(&self, id: LetterScriptNodeId) -> &LetterScriptNode {
        self.node_lookup.get(&id).unwrap()
    }

    pub(crate) fn register_node(
        &mut self,
        parent_id: LetterScriptNodeId,
        kind: LetterScriptNodeKind,
        span: SourceSpan,
    ) -> LetterScriptNodeId {
        let id = self.node_id_generator.next();
        let node = LetterScriptNode::new(id, kind, span);
        self.node_lookup.insert(id, node);
        self.node_lookup
            .get_mut(&parent_id)
            .unwrap()
            .register_child(id);

        id
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();

        let root_node = self.node_lookup.get(&self.root_id).unwrap();
        for child_id in root_node.children() {
            self.stringify_node(*child_id, &mut result, 0);
        }

        result
    }

    fn stringify_node(&self, node_id: LetterScriptNodeId, result: &mut String, indent: usize) {
        let node = self.node_lookup.get(&node_id).unwrap();
        let indent_string = " ".repeat(indent);

        self.stringify_node_start(node, result, &indent_string);

        for child_id in node.children() {
            self.stringify_node(*child_id, result, indent + 4);
        }

        self.stringify_node_end(node, result, &indent_string);
    }

    fn stringify_node_start(&self, node: &LetterScriptNode, result: &mut String, indent_str: &str) {
        result.push_str(indent_str);

        match node.kind() {
            LetterScriptNodeKind::Text(text) => {
                let lines: Vec<&str> = text.split('\n').collect();
                let line_count = lines.len();
                for (i, line) in lines.into_iter().enumerate() {
                    result.push_str(line);

                    let is_last_line = i == line_count - 1;
                    if !is_last_line {
                        result.push('\n');
                        result.push_str(indent_str);
                    }
                }
            }
            LetterScriptNodeKind::Heading => result.push_str("<heading>"),
            LetterScriptNodeKind::Paragraph => result.push_str("<paragraph>"),
            LetterScriptNodeKind::Section => result.push_str("<section>"),
            LetterScriptNodeKind::Image { src } => {
                result.push_str(&format!("<image src=\"{}\">", src));
            }
            LetterScriptNodeKind::Quote => result.push_str("<quote>"),
            LetterScriptNodeKind::List { ordered } => {
                if *ordered {
                    result.push_str("<list ordered=\"true\">");
                } else {
                    result.push_str("<list>");
                }
            }
            LetterScriptNodeKind::ListItem => result.push_str("<list-item>"),
            LetterScriptNodeKind::HorizontalRule => result.push_str("<horizontal-rule/>"),
            LetterScriptNodeKind::Link { target } => {
                result.push_str(&format!("<link target=\"{}\">", target))
            }
            LetterScriptNodeKind::Bold => result.push_str("<b>"),
            LetterScriptNodeKind::Italic => result.push_str("<i>"),
            LetterScriptNodeKind::Code { language } => {
                result.push_str("<code");

                if let Some(language) = language {
                    result.push_str(&format!(" language=\"{}\"", language));
                }

                result.push_str(">");
            }
            LetterScriptNodeKind::Table => result.push_str("<table>"),
            LetterScriptNodeKind::TableHeaderRow => result.push_str("<table-header-row>"),
            LetterScriptNodeKind::TableRow => result.push_str("<table-row>"),
            LetterScriptNodeKind::TableCell => result.push_str("<table-cell>"),
            LetterScriptNodeKind::Function { name, parameters } => {
                result.push_str(&format!("<{}", name));

                let mut entries = parameters.iter().collect::<Vec<_>>();
                entries.sort_by(|(key_a, _), (key_b, _)| key_a.cmp(key_b));
                for (key, value) in entries {
                    result.push_str(&format!(" {}=\"{}\"", key, value));
                }

                result.push_str(">");
            }
            _ => {}
        }

        result.push('\n');
    }

    fn stringify_node_end(&self, node: &LetterScriptNode, result: &mut String, indent_str: &str) {
        let s = match node.kind() {
            LetterScriptNodeKind::Heading => "</heading>".to_string(),
            LetterScriptNodeKind::Paragraph => "</paragraph>".to_string(),
            LetterScriptNodeKind::Section => "</section>".to_string(),
            LetterScriptNodeKind::Image { .. } => "</image>".to_string(),
            LetterScriptNodeKind::Quote => "</quote>".to_string(),
            LetterScriptNodeKind::List { .. } => "</list>".to_string(),
            LetterScriptNodeKind::ListItem => "</list-item>".to_string(),
            LetterScriptNodeKind::Link { .. } => "</link>".to_string(),
            LetterScriptNodeKind::Bold => "</b>".to_string(),
            LetterScriptNodeKind::Italic => "</i>".to_string(),
            LetterScriptNodeKind::Code { .. } => "</code>".to_string(),
            LetterScriptNodeKind::Table => "</table>".to_string(),
            LetterScriptNodeKind::TableHeaderRow => "</table-header-row>".to_string(),
            LetterScriptNodeKind::TableRow => "</table-row>".to_string(),
            LetterScriptNodeKind::TableCell => "</table-cell>".to_string(),
            LetterScriptNodeKind::Function { name, .. } => format!("</{}>", name),
            _ => "".to_string(),
        };

        if !s.is_empty() {
            result.push_str(indent_str);
            result.push_str(&s);
            result.push('\n');
        }
    }
}
