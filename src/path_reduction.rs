use std::{collections::BTreeMap, fmt::{Display, Debug}};

use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    convert::GNFA,
    extern_cfg::{BlockID, FunID},
    intern_cfg::CFG,
    re::RegExp,
};

pub struct PathReducer<BlockID, FunID> {
    res: BTreeMap<FunID, RegExp<BlockID, FunID>>,
    k: usize,
}

impl<BlockID: Eq + Clone + Debug, FunID: Eq + Clone + Ord + Display + Debug> PathReducer<BlockID, FunID> {
    pub fn reduce(&self, path: &[BlockID], cfg: FunID) -> Vec<BlockID> {
        let re = self.res.get(&cfg).expect("invalid fun_id");
        let (reduced_path, res) = re
            .parse_k(path, &self.res, self.k)
            .expect(&format!("ill structured path\nregex {:?}\n path {:?}", re, path));
        assert!(res.is_empty());
        reduced_path.into_vec()
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
            gnfa.reduce();
            let re = gnfa.start_to_end().clone();
            (fun_id, re)
        })
        .collect()
}
