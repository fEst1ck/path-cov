use std::{collections::BTreeMap, fmt::{Display, Debug}};

use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    convert::GNFA,
    extern_cfg::{BlockID, FunID},
    intern_cfg::CFG,
    re::{RegExp, ParseErr},
};
use petgraph::dot::{Dot};

pub struct PathReducer<BlockID, FunID> {
    res: BTreeMap<FunID, RegExp<BlockID, FunID>>,
    k: usize,
}

impl<BlockID: Eq + Clone + Debug, FunID: Eq + Clone + Ord + Debug> PathReducer<BlockID, FunID> {
    pub fn reduce(&self, path: &[BlockID], cfg: FunID) -> Vec<BlockID> {
        let re = self.res.get(&cfg).expect("invalid fun_id");
        match re.parse_k(path, &self.res, self.k) {
            Ok((reduced_path, res)) => {
                assert!(res.is_empty(), "there is a leftover of path {:?}", res);
                reduced_path.into_vec()
            }
            Err(ParseErr::Abort(val)) => {
                val.into_vec()
            }
            Err(ParseErr::Invalid) => {
                unreachable!("invalid path {:?}\n{:?}", path, self.res)
            }
        }
        // let (reduced_path, res) = re
        //     .parse_k(path, &self.res, self.k)
        //     .expect(&format!("ill structured path\nregex {:?}\n path {:?}", re, path));
        // assert!(res.is_empty(), "there is a leftover of path {:?}", res);
        // reduced_path.into_vec()
    }
}

impl PathReducer<BlockID, FunID> {
    pub fn from_cfgs(cfgs: BTreeMap<FunID, CFG<BlockID, FunID>>, k: usize) -> Self {
        let res = convert_cfgs(cfgs);
        Self { res, k }
    }
}

fn convert_cfgs(
    cfgs: BTreeMap<FunID, CFG<BlockID, FunID>>,
) -> BTreeMap<FunID, RegExp<BlockID, FunID>> {
    cfgs.into_iter()
        .par_bridge()
        .map(|(fun_id, cfg)| {
            let mut gnfa = GNFA::from_intern_cfg(cfg);
            println!("before reduce {:?}", Dot::new(&gnfa.the_graph));
            gnfa.reduce();
            println!("after reduce {:?}", Dot::new(&gnfa.the_graph));
            let re = gnfa.start_to_end().clone();
            (fun_id, re)
        })
        .collect()
}
