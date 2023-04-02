use std::collections::HashMap;

use self::node::{QuoteNode, QuoteNodeId};

mod node;

#[derive(Debug)]
pub(crate) struct QuoteTree {
    nodes: HashMap<QuoteNodeId, QuoteNode>,
    root: QuoteNodeId,
}
