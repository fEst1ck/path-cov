//! Control flow graph structure used in rust

use crate::convert::Node;

use petgraph::graph::{Graph, NodeIndex};

/// Control flow graph of a single function
#[derive(Debug, Clone)]
pub struct CFG<BlockID, FunID> {
    pub entry: NodeIndex,
    pub exit: NodeIndex,
    pub graph: Graph<Node<BlockID, FunID>, ()>,
}
