use std::{collections::BTreeMap, fmt::Debug};

use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    convert::GNFA,
    extern_cfg::{BlockID, FunID},
    intern_cfg::CFG,
    re::{RegExp, ParseErr},
};
use petgraph::dot::Dot;

pub struct PathReducer<BlockID, FunID> {
    res: BTreeMap<FunID, RegExp<BlockID, FunID>>,
    firsts: BTreeMap<BlockID, FunID>,
    k: usize,
}

impl<BlockID: Eq + Clone + Ord+ Debug, FunID: Eq + Clone + Ord + Debug> PathReducer<BlockID, FunID> {
    pub fn reduce(&self, mut path: &[BlockID], cfg: FunID) -> Vec<BlockID> {
        let unreduced_len = path.len();
        if path.is_empty() {
            return Vec::new();
        }
        let cfg = self.firsts.get(&path[0]).expect(&format!("no fun starts with {:?}", path[0]));
        let re = self.res.get(&cfg).expect("invalid fun_id");
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

                    panic!("invalid path {:?}\n{}", &path[0..100], s);
                    unreachable!("invalid path {:?}\n{:?}", &path[0..10], self.res)
                }
                
            }
        }
        // assert!(unreduced_len >= reduced_paths.len(), "unreduced_len: {}, reduced_len: {}", unreduced_len, reduced_paths.len());
        reduced_paths
    }
}

impl PathReducer<BlockID, FunID> {
    pub fn from_cfgs(cfgs: BTreeMap<FunID, CFG<BlockID, FunID>>, k: usize) -> Self {
        let res = convert_cfgs(cfgs);
        let mut firsts = BTreeMap::new();
        for (fun_id, re) in res.iter() {
            let first = re.first();
            if first == 36282 {
                println!("found!");
                // println!("fun_id: {:?}\n, re: {:?}", fun_id, re);
                // re.debug();
            }
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
