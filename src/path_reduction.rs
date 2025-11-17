use rustc_hash::{FxHashMap, FxHashSet};
use std::{env, fmt::Debug, hash::Hash};

use crate::{
    convert::GNFA,
    extern_cfg::{BlockID, FunID},
    intern_cfg::CFG,
    json_parser::parse_json_file,
    re::{ParseErr, RegExp},
};

pub struct PathReducer<BlockID, FunID> {
    res: FxHashMap<FunID, RegExp<BlockID, FunID>>,
    firsts: FxHashMap<BlockID, FunID>,
    lasts: FxHashMap<BlockID, FxHashSet<BlockID>>,
    k: usize,
}

impl<BlockID: Eq + Clone + Hash + Hash + Debug, FunID: Eq + Clone + Hash + Hash + Debug>
    PathReducer<BlockID, FunID>
{
    pub fn reduce(&self, mut path: &[BlockID], _cfg: FunID) -> Vec<BlockID> {
        self.simple_reduce(&mut path)
    }

    pub fn simple_reduce(&self, mut path: &[BlockID]) -> Vec<BlockID> {
        let mut res = Vec::new();
        while !path.is_empty() {
            let mut stack = vec![];
            res.append(&mut self.simple_reduce_one_fun(&mut path, &mut stack, false));
        }
        res
    }

    fn get_last_blocks(&self, block: &BlockID) -> Option<&FxHashSet<BlockID>> {
        // println!("get_last_blocks: {:?}", block);
        // println!("lasts: {:?}", self.lasts);
        self.lasts.get(block)
    }

    fn simple_reduce_one_fun(
        &self,
        path: &mut &[BlockID],
        stack: &mut Vec<BlockID>,
        skip: bool,
    ) -> Vec<BlockID> {
        // holds the reduced path of the current function call (including all sub-calls)
        let mut buffer = vec![];
        // maps a block to where it last appears in the buffer
        // this local to this function call
        let mut loop_stack: FxHashMap<BlockID, usize> = FxHashMap::default();
        let first = if let Some(first) = path.first() {
            first.clone()
        } else {
            return buffer;
        };
        // read the first block
        *path = &path[1..];
        stack.push(first.clone());
        if !skip {
            buffer.push(first.clone());
            loop_stack.insert(first.clone(), 0);
        }
        let lasts = if let Some(lasts) = self.get_last_blocks(&first) {
            lasts
        } else {
            return self.simple_reduce_one_fun(path, stack, skip)
        };
        if lasts.contains(&first) {
            // the function contains only one block
            // reach the end of the call
            while let Some(last) = stack.pop() {
                if last == first {
                    break;
                }
            }
            return buffer;
        }
        loop {
            if let Some(block) = path.first().cloned() {
                // block is the start of a new function
                if self.lasts.contains_key(&block) {
                    // the function is on stack
                    if skip || stack.iter().rev().find(|frame| frame == &&block).is_some() {
                        self.simple_reduce_one_fun(path, stack, true);
                    } else {
                        // reduce the path of this function call
                        buffer.append(&mut self.simple_reduce_one_fun(path, stack, skip));
                    }
                } else if lasts.contains(&block) {
                    // we reach the end of the current function call
                    *path = &path[1..];
                    if !skip {
                        buffer.push(block.clone());
                    }
                    while let Some(last) = stack.pop() {
                        if last == first {
                            break;
                        }
                    }
                    return buffer;
                } else {
                    // another block in the current function call
                    if skip {
                        *path = &path[1..];
                        continue;
                    }
                    // appears in the buffer at `last_off`
                    if let Some(&last_off) = loop_stack.get(&block) {
                        // remove the blocks starting from `last_off`
                        buffer.truncate(last_off);
                        loop_stack.retain(|_, &mut off| off < last_off);
                    }
                    *path = &path[1..];
                    buffer.push(block.clone());
                    loop_stack.insert(block.clone(), buffer.len() - 1);
                }
            } else {
                return buffer;
            }
        }
    }
}

impl PathReducer<BlockID, FunID> {
    pub fn from_cfgs(cfgs: FxHashMap<FunID, CFG<BlockID, FunID>>, k: usize) -> Self {
        let lasts = last_map(&cfgs);
        let res = convert_cfgs(cfgs);
        let mut firsts = FxHashMap::default();
        for (fun_id, re) in res.iter() {
            let first = re.first();
            let old = firsts.insert(first, fun_id.clone());
            if let Some(old_fun_id) = old {
                panic!(
                    "functions {} {} both start with block {}",
                    old_fun_id, fun_id, first
                );
            }
        }
        Self {
            res,
            firsts,
            lasts,
            k,
        }
    }
}

impl PathReducer<u32, u32> {
    pub fn from_json(path: &str) -> Self {
        let modules = parse_json_file(path).unwrap();
        let first_to_lasts = modules
            .iter()
            .flat_map(|module| {
                module.functions.iter().map(|func| {
                    (func.entry_block, func.exit_blocks.iter().cloned().collect())
                })
            })
            .collect();
        Self {
            res: FxHashMap::default(),
            firsts: FxHashMap::default(),
            lasts: first_to_lasts,
            k: 42,
        }
    }
}

fn convert_cfgs(
    cfgs: FxHashMap<FunID, CFG<BlockID, FunID>>,
) -> FxHashMap<FunID, RegExp<BlockID, FunID>> {
    cfgs.into_iter()
        .map(|(fun_id, cfg)| {
            let mut gnfa = GNFA::from_intern_cfg(cfg);
            gnfa.reduce();
            let re = gnfa.start_to_end().clone();
            (fun_id, re)
        })
        .collect()
}

/// Returns a map from the first block of a function to the set of exit blocks
fn last_map(
    cfgs: &FxHashMap<FunID, CFG<BlockID, FunID>>,
) -> FxHashMap<BlockID, FxHashSet<BlockID>> {
    cfgs.iter()
        // .par_bridge()
        .map(|(_fun_id, cfg)| {
            let first = cfg
                .graph
                .node_weight(cfg.entry)
                .unwrap()
                .clone()
                .to_block_id();
            let exit_node_indices: Vec<_> = cfg
                .graph
                .node_indices()
                .filter(|node_idx| cfg.graph.neighbors(*node_idx).count() == 0)
                .collect();
            let exit_nodes = exit_node_indices
                .iter()
                .map(|node_idx| {
                    cfg.graph
                        .node_weight(*node_idx)
                        .unwrap()
                        .clone()
                        .to_block_id()
                })
                .collect();
            (first, exit_nodes)
        })
        .collect()
}
