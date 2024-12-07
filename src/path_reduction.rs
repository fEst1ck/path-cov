use petgraph::algo::tarjan_scc;
use rustc_hash::{FxHashMap, FxHashSet};
use sha2::digest::block_buffer::Block;
use std::{env, fmt::Debug, hash::Hash};

use crate::{
    convert::GNFA,
    extern_cfg::{BlockID, FunID},
    intern_cfg::CFG,
    re::{ParseErr, RegExp},
};

const PATH_REDUCTION_DEBUG: &'static str = "PATH_REDUCTION_DEBUG";
const PATH_REDUCTION_ON_ERROR: &'static str = "PATH_REDUCTION_ON_ERROR";
const FULL_PATH: &'static str = "FULL_PATH";
const EMPTY_PATH: &'static str = "EMPTY_PATH";

pub struct PathReducer<BlockID, FunID> {
    res: FxHashMap<FunID, RegExp<BlockID, FunID>>,
    // firsts: FxHashMap<BlockID, FunID>,
    firsts: Vec<FunID>,
    // lasts: FxHashMap<BlockID, FxHashSet<BlockID>>,
    lasts: Vec<Option<FxHashSet<BlockID>>>,
    // maps a function (identified by its first block),
    // to the set of loop heads in the function
    loop_heads: FxHashMap<BlockID, FxHashSet<BlockID>>,
    k: usize,
}

// impl<BlockID: Eq + Clone + Hash + Hash + Debug, FunID: Eq + Clone + Hash + Hash + Debug>
impl
    PathReducer<BlockID, FunID>
{
    pub fn reduce(&self, mut path: &[BlockID], _cfg: FunID) -> Vec<BlockID> {
        if self.k == 42 {
            // println!("reducing path {:?}", path);
            let reduced = self.simple_reduce(&mut path);
            // println!("reduced path {:?}", reduced);
            return reduced;
        }
        let unreduced = path;
        if path.is_empty() {
            return Vec::new();
        }
        let cfg = self
            .firsts
            .get(path[0] as usize).unwrap();
            // .expect(&format!("no fun starts with {:?}", path[0]));
        // let re = self.res.get(&cfg).expect("invalid fun_id");
        let re = RegExp::Var(cfg.clone());
        let mut reduced_paths = Vec::new();
        while !path.is_empty() {
            match re.parse_k(path, &self.res, todo!(), self.k) {
                Ok((reduced_path, res)) => {
                    // assert!(res.len() < path.len());
                    let mut this_path = reduced_path.into_vec();
                    reduced_paths.append(&mut this_path);
                    path = res;
                }
                Err(ParseErr::Abort(val)) => {
                    reduced_paths.append(&mut val.into_vec());
                    return reduced_paths;
                }
                Err(ParseErr::Invalid(s)) => {
                    if let Ok(on_error) = env::var(PATH_REDUCTION_ON_ERROR) {
                        match on_error.as_str() {
                            FULL_PATH => {
                                if env::var(PATH_REDUCTION_DEBUG).is_ok() {
                                    println!("invalid path: {:?}", unreduced);
                                }
                                return unreduced.to_vec();
                            }
                            EMPTY_PATH => {
                                if env::var(PATH_REDUCTION_DEBUG).is_ok() {
                                    println!("invalid path: {:?}", unreduced);
                                }
                                return vec![];
                            }
                            _ => {
                                panic!("invalid value for PATH_REDUCTION_ON_ERROR: {}", on_error);
                            }
                        }
                    } else {
                        panic!("invalid path: {:?}, error: {}", unreduced, s);
                    }
                }
            }
        }
        reduced_paths
    }

    fn simple_reduce(&self, mut path: &[BlockID]) -> Vec<BlockID> {
        let mut res = Vec::with_capacity(1024); //vec![];
        while !path.is_empty() {
            let mut stack = FxHashSet::default();
            res.append(&mut self.simple_reduce_one_fun(&mut path, &mut stack, false));
            // println!("reduced one {:?}", reduced);
        }
        res
    }

    fn get_last_blocks(&self, block: &BlockID) -> &FxHashSet<BlockID> {
        &self.lasts[*block as usize].as_ref().unwrap()
            // .get(block)
            // .unwrap()
            // .expect(&format!("failed to get last blocks for block {:?}", block))
    }

    fn simple_reduce_one_fun(
        &self,
        path: &mut &[BlockID],
        stack: &mut FxHashSet<BlockID>,
        skip: bool,
    ) -> Vec<BlockID> {
        // holds the reduced path of the current function call (including all sub-calls)
        let mut buffer = Vec::with_capacity(1024); // Vec::new();
        // maps a block to where it last appears in the buffer
        // this local to this function call
        // let mut loop_stack: FxHashMap<BlockID, usize> = FxHashMap::default();
        let first = if let Some(first) = path.first() {
            first.clone()
        } else {
            return buffer;
        };
        let loop_heads = self
            .loop_heads
            .get(&first) // 1%
            .unwrap();
            // .expect(&format!("no loop heads for block {:?}", first));
        // read the first block
        *path = &path[1..];
        stack.insert(first.clone()); // 1%
        if !skip {
            buffer.push(first.clone());
            // loop_stack.insert(first.clone(), 0);
        }
        let lasts = self.get_last_blocks(&first); //TODO: 6%
        // println!("first {:?} lasts {:?}", first, lasts);
        if lasts.contains(&first) {
            // the function contains only one block
            // reach the end of the call
            if !skip {
                stack.remove(&first);
            }
            return buffer;
        }
        loop {
            if let Some(block) = path.first().cloned() {
                // block is the start of a new function
                if self.firsts[block as usize] != -1 { // TODO: 5% 6.7%
                    // the function is on stack
                    if skip || stack.contains(&block) {
                        self.simple_reduce_one_fun(path, stack, true);
                    } else {
                        // reduce the path of this function call
                        buffer.append(&mut self.simple_reduce_one_fun(path, stack, skip)); // TODO: 3% 4.27%
                        // self.simple_reduce_one_fun(path, stack, skip, &mut buffer);
                    }
                } else if lasts.contains(&block) { // TODO: 4% 4.5%
                    // we reach the end of the current function call
                    *path = &path[1..];
                    // stack.remove(&first);
                    if !skip {
                        stack.remove(&first);
                        buffer.push(block.clone()); // 1%
                        return buffer;
                    } else {
                        assert!(buffer.is_empty());
                        return buffer;
                    }
                } else {
                    // another block in the current function call
                    if skip {
                        *path = &path[1..];
                        continue;
                    }
                    if !loop_heads.contains(&block) {
                        *path = &path[1..];
                        buffer.push(block.clone()); // TODO: 2% 2.8%
                        continue;
                    }
                    // appears in the buffer at `last_off`
                    // if let Some(pos) = buffer.iter().rev().position(|x| x == &block) {
                    //     // block is not in the buffer
                    //     buffer.truncate(pos);
                    // }
                    Self::foo(&mut buffer, block.clone());
                    // if let Some(&last_off) = loop_stack.get(&block) {
                    //     // remove the blocks starting from `last_off`
                    //     // println!("before drain {:?}", buffer);
                    //     buffer.truncate(last_off);
                    //     // println!("after drain {:?}", buffer);
                    //     // loop_stack.retain(|_, &mut off| off < last_off);
                    // }
                    *path = &path[1..];
                    buffer.push(block.clone());
                    // if loop_heads.contains(&block) {
                    //     loop_stack.insert(block.clone(), buffer.len() - 1);
                    // }
                }
            } else {
                return buffer;
            }
        }
    }

    // #[inline(never)]
    fn foo(buffer: &mut Vec<BlockID>, block: BlockID) {
        if let Some(pos) = buffer.iter().rev().position(|x| x == &block) {
            // block is not in the buffer
            buffer.truncate(pos);
        }
    }
}

impl PathReducer<BlockID, FunID> {
    pub fn from_cfgs(cfgs: FxHashMap<FunID, CFG<BlockID, FunID>>, k: usize) -> Self {
        let lasts = last_map(&cfgs);

        let mut loop_heads = FxHashMap::default();

        for (fun_id, cfg) in cfgs.iter() {
            let graph = &cfg.graph;
            let components = tarjan_scc(graph);
            let mut heads = FxHashSet::default();
            for nodes in components.iter() {
                // println!("node {:?}", node.len());
                if nodes.len() == 1 {
                    continue;
                }

                for node in vec![nodes[0]] {
                    let weight = graph.node_weight(node).unwrap();
                    match weight {
                        crate::convert::Node::Literal(block) => {
                            heads.insert(block.clone());
                        }
                        crate::convert::Node::Var(_) => (),
                        crate::convert::Node::Extern => (),
                    }
                }
            }
            // if *fun_id == 346 {
            println!("heads {:?} out of {:?}", heads.len(), graph.node_count());
            // }
            loop_heads.insert(*fun_id, heads);
        }

        let res = convert_cfgs(cfgs);
        // let mut firsts = FxHashMap::default();
        let mut firsts = vec![-1; 1024 * 128];
        let mut fun_id_to_firsts = FxHashMap::default();
        for (fun_id, re) in res.iter() {
            let first = re.first();
            // let old = firsts.insert(first, fun_id.clone());
            let old_fun_id = firsts[first as usize];
            firsts[first as usize] = *fun_id;
            if old_fun_id != -1 {
                panic!(
                    "functions {} {} both start with block {}",
                    old_fun_id, fun_id, first
                );
            }
            fun_id_to_firsts.insert(*fun_id, first);

            // if *fun_id == 346 {
            //     println!("re: {:?}", re.size());
            // }
            // println!("computing loop heads for {:?}", fun_id);
            // loop_heads.insert(first, re.loop_heads());
        }
        Self {
            res,
            firsts,
            lasts,
            k,
            loop_heads: loop_heads
                .into_iter()
                .map(|(fun_id, heads)| {
                    let first = fun_id_to_firsts.get(&fun_id).unwrap();
                    (*first, heads)
                })
                .collect(),
        }
    }
}

fn convert_cfgs(
    cfgs: FxHashMap<FunID, CFG<BlockID, FunID>>,
) -> FxHashMap<FunID, RegExp<BlockID, FunID>> {
    cfgs.into_iter()
        // .par_bridge()
        .map(|(fun_id, cfg)| {
            let mut gnfa = GNFA::from_intern_cfg(cfg);
            // println!("before reduce {:?}", Dot::new(&gnfa.the_graph));
            gnfa.reduce();
            // println!("after reduce {:?}", Dot::new(&gnfa.the_graph));
            let re = gnfa.start_to_end().clone();
            (fun_id, re)
        })
        .collect()
}

/// Returns a map from the first block of a function to the set of exit blocks
fn last_map(
    cfgs: &FxHashMap<FunID, CFG<BlockID, FunID>>,
// ) -> FxHashMap<BlockID, FxHashSet<BlockID>> {
) -> Vec<Option<FxHashSet<BlockID>>> {
    let mut lasts = vec![None; 1024 * 128];
    for (_fun_id, cfg) in cfgs.iter() {
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
        lasts[first as usize] = Some(exit_nodes);
    }
    lasts


    // cfgs.iter()
    //     // .par_bridge()
    //     .map(|(_fun_id, cfg)| {
    //         let first = cfg
    //             .graph
    //             .node_weight(cfg.entry)
    //             .unwrap()
    //             .clone()
    //             .to_block_id();
    //         let exit_node_indices: Vec<_> = cfg
    //             .graph
    //             .node_indices()
    //             .filter(|node_idx| cfg.graph.neighbors(*node_idx).count() == 0)
    //             .collect();
    //         let exit_nodes = exit_node_indices
    //             .iter()
    //             .map(|node_idx| {
    //                 cfg.graph
    //                     .node_weight(*node_idx)
    //                     .unwrap()
    //                     .clone()
    //                     .to_block_id()
    //             })
    //             .collect();
    //         (first, exit_nodes)
    //     })
    //     .collect()
}
