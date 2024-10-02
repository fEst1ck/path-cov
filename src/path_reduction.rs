use std::{collections::BTreeMap, fmt::Debug, env};

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
    res: BTreeMap<FunID, RegExp<BlockID, FunID>>,
    firsts: BTreeMap<BlockID, FunID>,
    k: usize,
}

impl<BlockID: Eq + Clone + Ord+ Debug, FunID: Eq + Clone + Ord + Debug> PathReducer<BlockID, FunID> {
    pub fn reduce(&self, mut path: &[BlockID], _cfg: FunID) -> Vec<BlockID> {
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
}

impl PathReducer<BlockID, FunID> {
    pub fn from_cfgs(cfgs: BTreeMap<FunID, CFG<BlockID, FunID>>, k: usize) -> Self {
        let res = convert_cfgs(cfgs);
        let mut firsts = BTreeMap::new();
        for (fun_id, re) in res.iter() {
            let first = re.first();
            let old = firsts.insert(first, fun_id.clone());
            if let Some(old_fun_id) = old {
                panic!("functions {} {} both start with block {}", old_fun_id, fun_id, first);
            }
        }
        Self { res, firsts, k }
    }
}

fn convert_cfgs(
    cfgs: BTreeMap<FunID, CFG<BlockID, FunID>>,
) -> BTreeMap<FunID, RegExp<BlockID, FunID>> {
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
