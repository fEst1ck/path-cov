//! Structures of external C CFGs, and utilities for converting them to internal CFGs

use std::{
    collections::{BTreeMap, BTreeSet},
    os::raw::{c_char, c_int},
    slice,
};

use crate::{convert::Node, intern_cfg::CFG};
use petgraph::graph::Graph;
use petgraph::dot::Dot;


pub type FunID = c_int;
pub type BlockID = c_int;

const FUN_NAME_LEN: usize = 256;

#[repr(C)]
#[derive(Debug)]
struct CFGEntry {
    /// Name of the function, has length `FUN_NAME_LEN`
    function_name: [c_char; FUN_NAME_LEN],
    /// ID of the enery block
    entry: BlockID,
    /// ID of the exit block
    exit: BlockID,
}

#[repr(C)]
#[derive(Debug)]
struct BlockEntry {
    /// If the block is a call block,
    /// then the field contains the id of the function called,
    /// and -1 if the block is not a call block
    calls: FunID,
    /// Number of successors
    successor_size: c_int,
    /// Successor blocks
    successors_arr: *const BlockID,
}

#[repr(C)]
pub struct TopLevel {
    /// size of `cfg_arr`
    cfg_size: c_int,
    cfg_arr: *const CFGEntry,
    /// size of `block_arr`
    block_size: c_int,
    block_arr: *const *const BlockEntry,
}

/// Requires: `top_level` is not NULL
pub unsafe fn process_top_level(
    top_level: *const TopLevel,
) -> BTreeMap<FunID, CFG<BlockID, FunID>> {
    let top_level = top_level.as_ref().expect("top level");
    let cfgs = slice::from_raw_parts(top_level.cfg_arr, top_level.cfg_size as usize);
    let blocks = slice::from_raw_parts(top_level.block_arr, top_level.block_size as usize);
    let mut block_id_to_entry = BTreeMap::new();
    for (i, block) in blocks.iter().enumerate() {
        if !block.is_null() {
            let block_entry = &**block;
            block_id_to_entry.insert(i as BlockID, block_entry);
        }
    }
    process_cfgs(cfgs, &block_id_to_entry)
}

fn process_cfgs(cfgs: &[CFGEntry], blocks: &BTreeMap<BlockID, &BlockEntry>) -> BTreeMap<FunID, CFG<BlockID, FunID>> {
    cfgs.iter()
        .enumerate()
        .map(|(fun_id, cfg_entry)| (fun_id as FunID, process_cfg(cfg_entry, blocks)))
        .collect()
}

/// Returns the control flow graph of the given CFGEntry
fn process_cfg(cfg: &CFGEntry, blocks: &BTreeMap<BlockID, &BlockEntry>) -> CFG<BlockID, FunID> {
    // println!("cfg {:?}\n blocks {:?}", cfg, blocks);
    let entry_block_id = cfg.entry;
    let exit_block_id = cfg.exit;
    let res = get_cfg_with_root(entry_block_id, exit_block_id, blocks);
    // println!("graph {:?}", Dot::new(&res.graph));
    res
}

/// Given the block entries indexed by `BlockID`,
/// returns the control flow graph with root `entry`
fn get_cfg_with_root(entry: BlockID, exit: BlockID, blocks: &BTreeMap<BlockID, &BlockEntry>) -> CFG<BlockID, FunID> {
    let mut graph = Graph::new();
    let mut block_id_to_node_idx = BTreeMap::new();
    // add node to graph for each block
    for block_id in DFS::new(blocks, entry) {
        let block_entry = blocks.get(&block_id).expect("invalid block id");
        let node_weight = if block_entry.calls == -1 {
            Node::Literal(block_id)
        } else if block_entry.calls == -2 {
            Node::Extern
        } else {
            debug_assert!(
                block_entry.successor_size >= 0,
                "call block {} has {} successors",
                block_id,
                block_entry.successor_size
            );
            Node::Var(block_entry.calls)
        };
        let node_idx = graph.add_node(node_weight);
        let no_dup = block_id_to_node_idx.insert(block_id, node_idx).is_none();
        debug_assert!(no_dup, "duplicate block id{}", block_id);
    }
    // add edges to the graph
    for block_id in DFS::new(blocks, entry as BlockID) {
        let node_idx = *block_id_to_node_idx.get(&block_id).unwrap();
        for succ_block in get_successors(blocks, block_id).iter().cloned().collect::<BTreeSet<_>>().into_iter() {
            if block_id == 36290 {
                println!("block_id: {}, succ_block: {}", block_id, succ_block);
            }
            let succ_node_idx = *block_id_to_node_idx.get(&succ_block).unwrap();
            graph.add_edge(node_idx, succ_node_idx, ());
        }
    }
    CFG {
        entry: *block_id_to_node_idx.get(&entry).expect("entry block idx"),
        exit: *block_id_to_node_idx.get(&exit).expect("entry block idx"),
        graph,
    }
}

/// Given the block entries indexed by `BlockID`,
/// returns the id of the successor blocks of the given block
fn get_successors<'a>(blocks: &'a BTreeMap<BlockID, &'a BlockEntry>, block_id: BlockID) -> &'a [BlockID] {
    let block_entry = blocks.get(&block_id).expect("invalid block id");
    unsafe {
        slice::from_raw_parts(
            block_entry.successors_arr,
            block_entry.successor_size as usize,
        )
    }
}

/// State for DFS traversal of the CFG
struct DFS<'a> {
    to_visit: Vec<BlockID>,
    visited: BTreeSet<BlockID>,
    blocks: &'a BTreeMap<BlockID, &'a BlockEntry>,
}

impl<'a> DFS<'a> {
    /// Traverse the CFG with root `entry`
    fn new(blocks: &'a BTreeMap<BlockID, &BlockEntry>, entry: BlockID) -> Self {
        Self {
            to_visit: vec![entry],
            visited: BTreeSet::new(),
            blocks,
        }
    }

    /// Gets the next scheduled unvisited block
    fn get_next_unvisited(&mut self) -> Option<BlockID> {
        while let Some(next_scheduled) = self.to_visit.pop() {
            if !self.visited.contains(&next_scheduled) {
                return Some(next_scheduled);
            }
        }
        None
    }
}

impl<'a> Iterator for DFS<'a> {
    type Item = BlockID;

    fn next(&mut self) -> Option<Self::Item> {
        let next_unvisited = self.get_next_unvisited()?;
        self.visited.insert(next_unvisited);
        let block = self
            .blocks
            .get(&(next_unvisited as BlockID))
            .expect("invalid block id");
        for suc_block_id in
            unsafe { slice::from_raw_parts(block.successors_arr, block.successor_size as usize) }
        {
            self.to_visit.push(*suc_block_id);
        }
        Some(next_unvisited)
    }
}
