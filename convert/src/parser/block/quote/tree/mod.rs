use std::collections::HashMap;

use self::node::{QuoteNode, QuoteNodeId};

mod node;

pub(crate) struct QuoteTree {
    nodes: HashMap<QuoteNodeId, QuoteNode>,
    root: QuoteNodeId,
}
