//! AST for the textual cfg, and utilities to convert the ast to a CFG in graph representation.

use std::collections::{btree_map::Entry, BTreeMap};
use crate::convert::Node as Node;

use petgraph::graph::Graph;

#[derive(Debug)]
pub struct CFG<BlockID, FunID> {
    pub(crate) id: FunID,
    pub(crate) block_entries: Vec<BlockEntry<BlockID, FunID>>,
}

impl<BlockID: Clone + Ord, FunID: Clone> CFG<BlockID, FunID> {
	pub fn new(self) -> (FunID, Graph<Node<BlockID, FunID>, ()>) {
        let mut graph = Graph::new();
        let mut map = BTreeMap::new();
		// creates a new node for each block entry, and maps block id to node idx
        for block_entry in &self.block_entries {
            match map.entry(block_entry.id.clone()) {
                Entry::Vacant(e) => {
					let node_idx = graph.add_node(
						match &block_entry.calls {
							Some(fun_id) => Node::Var(fun_id.clone()),
							None => Node::Literal(block_entry.id.clone()),
						}
					);
					e.insert(node_idx);
				},
                Entry::Occupied(..) => {
					panic!("duplicate block entry")
				},
            }
        }
        for BlockEntry {
            id,
            calls: _,
            successors,
        } in self.block_entries
        {
			let from = map.get(&id).expect("node index");
			for block in successors {
				let to = map.get(&block).expect("node index");
				graph.add_edge(*from, *to, ());
			}
		}
        (self.id, graph)
    }
}

/// Contains a block id, its successors blocks,
/// and optionally a function id if the block is a call block
#[derive(Debug)]
pub struct BlockEntry<BlockID, FunID> {
    pub(crate) id: BlockID,
    pub(crate) calls: Option<FunID>,
    pub(crate) successors: Vec<BlockID>,
}
