use std::{collections::{HashMap, HashSet}, env, fmt::Debug, hash::Hash};

use crate::{
    convert::GNFA,
    extern_cfg::{BlockID, FunID},
    intern_cfg::CFG,
    re::{RegExp, ParseErr},
};

const PATH_REDUCTION_DEBUG: &'static str = "PATH_REDUCTION_DEBUG";
const PATH_REDUCTION_ON_ERROR: &'static str = "PATH_REDUCTION_ON_ERROR";
const FULL_PATH : &'static str = "FULL_PATH";
const EMPTY_PATH : &'static str = "EMPTY_PATH";

pub struct PathReducer<BlockID, FunID> {
    res: HashMap<FunID, RegExp<BlockID, FunID>>,
    firsts: HashMap<BlockID, FunID>,
    lasts: HashMap<BlockID, HashSet<BlockID>>,
    k: usize,
}

impl<BlockID: Eq + Clone + Hash + Hash + Debug, FunID: Eq + Clone + Hash + Hash + Debug> PathReducer<BlockID, FunID> {
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
        let cfg = self.firsts.get(&path[0]).expect(&format!("no fun starts with {:?}", path[0]));
        // let re = self.res.get(&cfg).expect("invalid fun_id");
        let re = RegExp::Var(cfg.clone());
        let mut reduced_paths = Vec::new();
        while !path.is_empty() {
            match re.parse_k(path, &self.res, &self.firsts, self.k) {
                Ok((reduced_path, res)) => {
                    // assert!(res.len() < path.len());
                    let mut this_path = reduced_path.into_vec();
                    reduced_paths.append(&mut this_path);
                    path = res;
                }
                Err(ParseErr::Abort(val)) => {
                    reduced_paths.append(&mut val.into_vec());
                    return reduced_paths
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
        let mut res = Vec::new();
        while !path.is_empty() {
            let mut stack = vec![];
            self.simple_reduce_one_fun(&mut path, &mut stack, false, &mut res);
            // println!("reduced one {:?}", reduced);
        }
        res
    }

    fn get_last_blocks(&self, block: &BlockID) -> &HashSet<BlockID> {
        self.lasts.get(block).expect(&format!("failed to get last blocks for block {:?}", block))
    }

    fn simple_reduce_one_fun(&self, path: &mut &[BlockID], stack: &mut Vec<BlockID>, skip: bool, out: &mut Vec<BlockID>) {
        // holds the reduced path of the current function call (including all sub-calls)
        let mut buffer = Vec::new();
        // maps a block to where it last appears in the buffer
        // this local to this function call
        let mut loop_stack: HashMap<BlockID, usize> = HashMap::new();
        let first = if let Some(first) = path.first() {
            first.clone()
        } else {
            return;
        };
        if skip {
            // println!("skipping {:?}", first);
        } else {
            // println!("reducing {:?}", first);
        }
        // read the first block
        *path = &path[1..];
        stack.push(first.clone());
        if !skip {
            buffer.push(first.clone());
            loop_stack.insert(first.clone(), 0);
        }
        let lasts = self.get_last_blocks(&first);
        // println!("first {:?} lasts {:?}", first, lasts);
        if lasts.contains(&first) {
            // the function contains only one block
            // reach the end of the call
            while let Some(last) = stack.pop() {
                if last == first {
                    break;
                }
            }
            out.append(&mut buffer);
            return;
            // return buffer;
        }
        loop {
            if let Some(block) = path.first().cloned() {
                // block is the start of a new function
                if self.firsts.contains_key(&block) {
                    // the function is on stack
                    if skip || stack.iter().rev().find(|frame| frame == &&block).is_some() {
                        self.simple_reduce_one_fun(path, stack, true, out);
                    } else {
                        // reduce the path of this function call
                        // buffer.append(&mut self.simple_reduce_one_fun(path, stack, skip));
                        self.simple_reduce_one_fun(path, stack, skip, &mut buffer);
                    }
                } else if lasts.contains(&block) { // we reach the end of the current function call
                    *path = &path[1..];
                    // stack.remove(&first);
                    while let Some(last) = stack.pop() {
                        if last == first {
                            break;
                        }
                    }
                    if !skip {
                        // since we return immediately, we don't need to update the loop stack
                        buffer.push(block.clone());
                    }
                    out.append(&mut buffer);
                    return;
                    // return buffer;
                } else { // another block in the current function call
                    if skip {
                        *path = &path[1..];
                        continue;
                    }
                    // appears in the buffer at `last_off`
                    if let Some(&last_off) = loop_stack.get(&block) {
                        // remove the blocks starting from `last_off`
                        // println!("before drain {:?}", buffer);
                        buffer.truncate(last_off);
                        // println!("after drain {:?}", buffer);
                        loop_stack.retain(|_, &mut off| off < last_off);
                    }
                    *path = &path[1..];
                    buffer.push(block.clone());
                    loop_stack.insert(block.clone(), buffer.len() - 1);
                }
            } else {
                // the current function call aborts
                out.append(&mut buffer);
                return;
                // return buffer;
            }
        }
    }
}

impl PathReducer<BlockID, FunID> {
    pub fn from_cfgs(cfgs: HashMap<FunID, CFG<BlockID, FunID>>, k: usize) -> Self {
        let lasts = last_map(&cfgs);
        let res = convert_cfgs(cfgs);
        let mut firsts = HashMap::new();
        for (fun_id, re) in res.iter() {
            let first = re.first();
            let old = firsts.insert(first, fun_id.clone());
            if let Some(old_fun_id) = old {
                panic!("functions {} {} both start with block {}", old_fun_id, fun_id, first);
            }
        }
        Self { res, firsts, lasts, k }
    }
}

fn convert_cfgs(
    cfgs: HashMap<FunID, CFG<BlockID, FunID>>,
) -> HashMap<FunID, RegExp<BlockID, FunID>> {
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
    cfgs: &HashMap<FunID, CFG<BlockID, FunID>>,
) -> HashMap<BlockID, HashSet<BlockID>> {
    cfgs.iter()
        // .par_bridge()
        .map(|(_fun_id, cfg)| {
            let first = cfg.graph.node_weight(cfg.entry).unwrap().clone().to_block_id();
            let exit_node_indices: Vec<_> = cfg.graph.node_indices().filter(|node_idx| cfg.graph.neighbors(*node_idx).count() == 0).collect();
            let exit_nodes = exit_node_indices.iter().map(|node_idx| cfg.graph.node_weight(*node_idx).unwrap().clone().to_block_id()).collect();
            (first, exit_nodes)
        })
        .collect()
}
