//! Regular expressions

use core::panic;
use std::{
    collections::BTreeMap,
    fmt::{format, Debug},
    sync::Arc,
};

/// Regular expressions over alphabet set `Alphabet`, and variable set `Name`
/// a variable refers to an external regular expression
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegExp<Alphabet, Name> {
    Epsilon,
    Var(Name),
    Literal(Alphabet),
    Literals(Vec<Alphabet>),
    Concat(Arc<RegExp<Alphabet, Name>>, Arc<RegExp<Alphabet, Name>>),
    Seq(Vec<Arc<RegExp<Alphabet, Name>>>),
    Alter(Arc<RegExp<Alphabet, Name>>, Arc<RegExp<Alphabet, Name>>),
    Star(Arc<RegExp<Alphabet, Name>>),
}

#[derive(Debug)]
pub enum ParseErr<Alphabet> {
    Abort(Val<Alphabet>),
    Invalid(String),
}

impl<Alphabet: Eq + Clone + Ord + Debug, Name: Eq + Clone + Ord + Debug> RegExp<Alphabet, Name> {
    pub fn size(&self) {
        match self {
            RegExp::Epsilon | RegExp::Var(_) | RegExp::Literal(_) => (),
            RegExp::Literals(vs) => (),
            RegExp::Concat(r1, r2) => {
                r1.size();
                r2.size();
            }
            RegExp::Seq(rs) => {
                for r in rs {
                    r.size();
                }
            }
            RegExp::Alter(r1, r2) => {
                r1.size();
                r2.size();
            }
            RegExp::Star(r) => r.size(),
        }
    }

    pub fn debug(&self) {
        match self {
            RegExp::Epsilon => println!("Epsilon"),
            RegExp::Var(x) => println!("Var({:?})", x),
            RegExp::Literal(lit) => println!("Literal({:?})", lit),
            RegExp::Literals(lits) => println!("Literals({:?})", lits),
            RegExp::Concat(x, y) => {
                println!("Concat(");
                x.debug();
                println!(",");
                y.debug();
                println!(")");
            }
            RegExp::Seq(_) => todo!(),
            RegExp::Alter(x, y) => {
                println!("Alter(");
                x.debug();
                println!(",");
                y.debug();
                println!(")");
            }
            RegExp::Star(x) => {
                println!("Star(");
                x.debug();
                println!(")");
            }
        }
    }

    pub fn first(&self) -> Alphabet {
        // match self {
        //     RegExp::Epsilon => panic!("first: start with epsilon"),
        //     RegExp::Var(_) => panic!("first: start with var"),
        //     RegExp::Literal(c) => c.clone(),
        //     RegExp::Literals(cs) => cs[0].clone(),
        //     RegExp::Concat(re, _) => re.first(),
        //     RegExp::Seq(res) => res[0].first(),
        //     RegExp::Alter(re1, re2) => {
        //         let f1 = re1.first();
        //         let f2 = re2.first();
        //         if f1 == f2 {
        //             f1
        //         } else {
        //             panic!("first: may begin with {:?} {:?}", f1, f2)
        //         }
        //     },
        //     RegExp::Star(re) => re.first(),
        // }
        self.first_opt().unwrap()
    }

    pub fn first_opt(&self) -> Option<Alphabet> {
        match self {
            RegExp::Epsilon => None,
            RegExp::Var(x) => panic!("first: start with var {:?}", x),
            RegExp::Literal(c) => Some(c.clone()),
            RegExp::Literals(cs) => Some(cs[0].clone()),
            RegExp::Concat(re1, re2) => {
                if let Some(c) = re1.first_opt() {
                    Some(c)
                } else {
                    re2.first_opt()
                }
            }
            RegExp::Seq(res) => todo!(),
            RegExp::Alter(re1, re2) => {
                let f1 = re1.first_opt();
                if f1.is_some() {
                    return f1;
                }
                let f2 = re2.first_opt();
                f2
            }
            RegExp::Star(re) => re.first_opt(),
        }
    }

    #[allow(dead_code)]
    pub fn var(x: Name) -> Self {
        Self::Var(x)
    }

    #[allow(dead_code)]
    pub fn literal(c: Alphabet) -> Self {
        Self::Literal(c)
    }

    pub fn concat(r1: Arc<RegExp<Alphabet, Name>>, r2: Arc<RegExp<Alphabet, Name>>) -> Arc<Self> {
        use RegExp::*;
        match (r1.as_ref(), r2.as_ref()) {
            (Epsilon, _r) => r2,
            (_r, Epsilon) => r1,
            // (Literal(l1), Literal(l2)) => Arc::new(Literals(vec![l1.clone(), l2.clone()])),
            // (Literal(l1), Literals(mut l2)) => {
            //     let mut res = vec![l1.clone()];
            //     res.append(&mut l2);
            //     Arc::new(Literals(res))
            // }
            // (Literals(mut l1), Literal(l2)) => {
            //     l1.push(l2.clone());
            //     Arc::new(Literals(l1))
            // }
            // (Literals(mut l1), Literals(mut l2)) => {
            //     l1.append(&mut l2);
            //     Arc::new(Literals(l1))
            // }
            // (Seq(mut es1), Seq(mut es2)) => {
            //     es1.append(&mut es2);
            //     Arc::new(Seq(es1))
            // }
            // (Seq(mut es1), _r2) => {
            //     es1.push(r2);
            //     Arc::new(Seq(es1))
            // }
            // (_r1, Seq(mut es2)) => {
            //     let mut es = vec![r1];
            //     es.append(&mut es2);
            //     Arc::new(Seq(es))
            // }
            // (_r1, _r2) => Arc::new(Seq(vec![r1, r2])),
            (_, _) => Arc::new(RegExp::Concat(r1, r2)),
        }
        //Self::Concat(Arc::new(r1), Arc::new(r2))
    }

    pub fn alter(r1: Arc<Self>, r2: Arc<Self>) -> Arc<Self> {
        assert!(!matches!(r1.as_ref(), RegExp::Epsilon));
        assert!(!matches!(r2.as_ref(), RegExp::Epsilon));
        if r1 == r2 {
            return r1;
        }
        // let (prefix, r1, r2) = Self::alter_prefix_acc(Arc::new(RegExp::Epsilon), r1, r2);
        // let (r1, r2, postfix) = Self::alter_post_acc(r1, r2, Arc::new(RegExp::Epsilon));
        // RegExp::concat(
        //     prefix,
        //     RegExp::concat(Arc::new(RegExp::Alter(r1, r2)), postfix),
        // )
        Arc::new(RegExp::Alter(r1, r2))
    }

    pub fn alter_prefix_acc(
        prefix: Arc<Self>,
        r1: Arc<Self>,
        r2: Arc<Self>,
    ) -> (Arc<Self>, Arc<Self>, Arc<Self>) {
        assert!(!matches!(r1.as_ref(), RegExp::Epsilon));
        assert!(!matches!(r2.as_ref(), RegExp::Epsilon));
        use RegExp::*;
        let epsilon = Arc::new(Epsilon);
        match (r1.as_ref(), r2.as_ref()) {
            (_r1, _r2) if r1 == r2 => {
                (RegExp::concat(prefix, r1), epsilon.clone(), epsilon.clone())
            }
            // (Literal(l1), Literal(l2)) if l1 == l2 => (RegExp::concat(prefix, Literal(l1)), epsilon, epsilon),
            // (Var(x), Var(y)) if x == y => (RegExp::concat(prefix, Var(x)), epsilon, epsilon),
            (Concat(a, b), Concat(c, d)) => {
                let (p1, a_, c_) = Self::alter_prefix_acc(prefix, a.clone(), c.clone());
                if matches!(a_.as_ref(), Epsilon) && matches!(c_.as_ref(), Epsilon) {
                    Self::alter_prefix_acc(p1, b.clone(), d.clone())
                } else {
                    (
                        p1,
                        RegExp::concat(a_, b.clone()),
                        RegExp::concat(c_, d.clone()),
                    )
                }
            }
            (_r1, _r2) => (prefix, r1, r2),
        }
    }

    pub fn alter_post_acc(
        r1: Arc<Self>,
        r2: Arc<Self>,
        postfix: Arc<Self>,
    ) -> (Arc<Self>, Arc<Self>, Arc<Self>) {
        assert!(!matches!(r1.as_ref(), RegExp::Epsilon));
        assert!(!matches!(r2.as_ref(), RegExp::Epsilon));
        use RegExp::*;
        let epsilon = Arc::new(Epsilon);
        match (r1.as_ref(), r2.as_ref()) {
            (r1_, r2_) if r1 == r2 => (epsilon.clone(), epsilon, RegExp::concat(r1, postfix)),
            (Concat(a, b), Concat(c, d)) => {
                let (b_, d_, p1) = Self::alter_post_acc(b.clone(), d.clone(), postfix);
                if matches!(b_.as_ref(), Epsilon) && matches!(d_.as_ref(), Epsilon) {
                    Self::alter_post_acc(a.clone(), c.clone(), p1)
                } else {
                    (
                        RegExp::concat(a.clone(), b_),
                        RegExp::concat(c.clone(), d_),
                        p1,
                    )
                }
            }
            (_r1, _r2) => (r1, r2, postfix),
        }
    }

    pub fn star(r: Arc<Self>) -> Self {
        Self::Star(r)
    }

    #[allow(dead_code)]
    pub fn parse_inf<'a>(
        &self,
        s: &'a [Alphabet],
        env: &BTreeMap<Name, RegExp<Alphabet, Name>>,
    ) -> Option<(Val<Alphabet>, &'a [Alphabet])> {
        match self {
            RegExp::Epsilon => todo!(),
            RegExp::Var(x) => {
                let re = env.get(x).expect("name {x} doesn't exist in env");
                re.parse_inf(s, env)
            }
            RegExp::Literal(c) => {
                if s.is_empty() {
                    None
                } else {
                    if c == &s[0] {
                        Some((Val::Literal(c.clone()), &s[1..]))
                    } else {
                        None
                    }
                }
            }
            RegExp::Concat(r1, r2) => {
                let (v1, s1) = r1.parse_inf(s, env)?;
                let (v2, s2) = r2.parse_inf(s1, env)?;
                Some((Val::Concat(Box::new(v1), Box::new(v2)), s2))
            }
            RegExp::Alter(r1, r2) => match r1.parse_inf(s, env) {
                Some(res) => Some(res),
                None => r2.parse_inf(s, env),
            },
            RegExp::Star(r) => {
                let (vs, s1) = r.parse_star_inf(s, env);
                Some((Val::Star(vs), s1))
            }
            RegExp::Literals(_) => todo!(),
            RegExp::Seq(_) => todo!(),
        }
    }

    pub fn parse_k<'a>(
        &self,
        s: &'a [Alphabet],
        env: &BTreeMap<Name, RegExp<Alphabet, Name>>,
        firsts: &BTreeMap<Alphabet, Name>,
        k: usize,
    ) -> Result<(Val<Alphabet>, &'a [Alphabet]), ParseErr<Alphabet>> {
        let mut stack = BTreeMap::new();
        let mut matched = 0;
        self._parse_k(s, env, firsts, k, &mut stack, &mut matched)
    }

    pub fn _parse_k<'a>(
        &self,
        s: &'a [Alphabet],
        env: &BTreeMap<Name, RegExp<Alphabet, Name>>,
        firsts: &BTreeMap<Alphabet, Name>,
        k: usize,
        stack: &mut BTreeMap<Name, usize>,
        matched: &mut usize,
    ) -> Result<(Val<Alphabet>, &'a [Alphabet]), ParseErr<Alphabet>> {
        // if !s.is_empty() {
        //     println!("matched so far: {:?} to match {:?}", matched, &s[0]);
        // }
        // println!("_parse_k\nre:{:?}\ns:{:?}", self.first(), s);
        // println!("s:{:?}", &s[..10]);
        match self {
            RegExp::Epsilon => Ok((Val::Star(Vec::new()), s)),
            RegExp::Var(x) => {
                // println!("var");
                let re = env
                    .get(x)
                    .expect(&format!("name {:?} doesn't exist in env", x));
                let nested_level = *stack.entry(x.clone()).or_default();
                if nested_level == k {
                    match re._parse_k(s, env, firsts, k, stack, matched) {
                        Ok((_, s)) => Ok((Val::Epsilon, s)),
                        Err(ParseErr::Abort(_)) => Err(ParseErr::Abort(Val::Epsilon)),
                        res @ Err(ParseErr::Invalid(_)) => res,
                    }
                } else {
                    *stack.get_mut(x).unwrap() += 1;
                    let res = re._parse_k(s, env, firsts, k, stack, matched);
                    *stack.get_mut(x).unwrap() -= 1;
                    res
                }
            }
            RegExp::Literal(c) => {
                // println!("literal {:?}", c);
                if s.is_empty() {
                    Err(ParseErr::Abort(Val::Epsilon))
                } else {
                    if c == &s[0] {
                        // println!("matched {:?}", c);
                        *matched += 1;
                        // println!("matched so far: {:?}", matched);
                        Ok((Val::Literal(c.clone()), &s[1..]))
                    } else {
                        if let Some(x) = firsts.get(&s[0]) {
                            // println!("implicit call!");
                            let re = RegExp::Var(x.clone());
                            let (val, s1) = re._parse_k(s, env, firsts, k, stack, matched)?;
                            match RegExp::Literal(c.clone())._parse_k(s1, env, firsts, k, stack, matched) {
                                Ok((val2, s2)) => Ok((Val::Concat(Box::new(val), Box::new(val2)), s2)),
                                Err(ParseErr::Abort(x)) => Err(ParseErr::Abort(Val::Concat(Box::new(val), Box::new(x)))),
                                res @ Err(ParseErr::Invalid(_)) => res,
                            }
                            // Err(ParseErr::Abort(Val::Epsilon))
                            // res
                            // match res {
                            //     Ok((val, s)) => todo!(),
                            //     Err(_) => todo!(),
                            // }
                        } else {
                            // println!("expected {:?} found {:?} stack: {:?}", c, &s, &stack);
                            // println!("no function starts with {:?}", &s[0]);
                            // println!("expected {:?} found {:?}", c, &s[0]);
                            // println!("firsts: {:?}", firsts);
                            // Err(ParseErr::Invalid(format!("no function starts with {:?}", &s[0])))
                            Err(ParseErr::Invalid(format!(
                                "expected {:?} found {:?}",
                                c, &s[0]
                            )))
                        }
                    }
                }
            }
            RegExp::Literals(lits) => {
                let mut lit_vals = Vec::new();
                let mut rest = s;
                for lit in lits {
                    if rest.is_empty() {
                        return Err(ParseErr::Abort(Val::Seq(lit_vals)));
                    } else {
                        if lit == &rest[0] {
                            lit_vals.push(Val::Literal(lit.clone()));
                            rest = &rest[1..];
                        } else {
                            if let Some(x) = firsts.get(&rest[0]) {
                                let re = RegExp::Var(x.clone());
                                let res = re._parse_k(s, env, firsts, k, stack, matched);
                                match res {
                                    Ok((val, rest_path)) => {
                                        lit_vals.push(val);
                                        rest = rest_path;
                                    }
                                    Err(ParseErr::Abort(v)) => {
                                        lit_vals.push(v);
                                        return Err(ParseErr::Abort(Val::Seq(lit_vals)));
                                    }
                                    res @ Err(ParseErr::Invalid(_)) => return res,
                                }
                            } else {
                                // println!("expected {:?} found {:?} stack: {:?}", c, &s, &stack);
                                // println!("expected {:?} found {:?}", lit, &rest[0]);
                                // println!("firsts: {:?}", firsts);
                                return Err(ParseErr::Invalid(format!(
                                    "expected {:?} found {:?}",
                                    lit, &rest[0]
                                )));
                            }
                        }
                    }
                }
                Ok((Val::Seq(lit_vals), rest))
            }
            RegExp::Concat(r1, r2) => {
                // println!("concat");
                // println!("concat1 {:?}", r1);
                let (v1, s1) = r1._parse_k(s, env, firsts, k, stack, matched)?;
                // println!("concat2 {:?}", r1);
                match r2._parse_k(s1, env, firsts, k, stack, matched) {
                    Ok((v2, s2)) => Ok((Val::Concat(Box::new(v1), Box::new(v2)), s2)),
                    Err(ParseErr::Abort(v2)) => {
                        Err(ParseErr::Abort(Val::Concat(Box::new(v1), Box::new(v2))))
                    }
                    res @ Err(ParseErr::Invalid(_)) => res,
                }
            }
            RegExp::Seq(rs) => {
                let mut vals = Vec::new();
                let mut rest = s;
                for r in rs {
                    match r._parse_k(rest, env, firsts, k, stack, matched) {
                        Ok((v, s)) => {
                            vals.push(v);
                            rest = s;
                        }
                        Err(ParseErr::Abort(v)) => {
                            vals.push(v);
                            return Err(ParseErr::Abort(Val::Seq(vals)));
                        }
                        res @ Err(ParseErr::Invalid(_)) => {
                            return res;
                        }
                    }
                }
                Ok((Val::Seq(vals), rest))
            }
            RegExp::Alter(r1, r2) => {
                // println!("alter");
                let cur = *matched;
                // println!("alter1");
                match r1._parse_k(s, env, firsts, k, stack, matched) {
                    res @ Ok(..) | res @ Err(ParseErr::Abort(..)) => res,
                    Err(ParseErr::Invalid(msg)) => {
                        // println!("r1 invalid: {:?}", msg);
                        *matched = cur;
                        // println!("alter2");
                        r2._parse_k(s, env, firsts, k, stack, matched)
                    }
                }
            }
            RegExp::Star(r) => {
                // println!("star");
                match r.parse_star_k(s, env, firsts, k, stack, matched) {
                Ok((vals, s)) => Ok((Val::Star(vals), s)),
                Err(ParseErr::Abort(val)) => Err(ParseErr::Abort(val)),
                Err(ParseErr::Invalid(s)) => Err(ParseErr::Invalid(s)),
            }},
        }
    }

    #[allow(dead_code)]
    fn parse_star_inf<'a>(
        &self,
        mut s: &'a [Alphabet],
        env: &BTreeMap<Name, Self>,
    ) -> (Vec<Val<Alphabet>>, &'a [Alphabet]) {
        let mut acc = Vec::new();
        while let Some((val, new_s)) = self.parse_inf(s, env) {
            s = new_s;
            acc.push(val);
        }
        (acc, s)
    }

    fn parse_star_k<'a>(
        &self,
        mut s: &'a [Alphabet],
        env: &BTreeMap<Name, Self>,
        firsts: &BTreeMap<Alphabet, Name>,
        k: usize,
        stack: &mut BTreeMap<Name, usize>,
        matched: &mut usize,
    ) -> Result<(Vec<Val<Alphabet>>, &'a [Alphabet]), ParseErr<Alphabet>> {
        let mut acc = Vec::new();
        loop {
            let cur = *matched;
            match self._parse_k(s, env, firsts, k, stack, matched) {
                Ok((val, new_s)) => {
                    s = new_s;
                    if acc.len() == k {
                        // consumes more `self`, but don't push to `acc`
                        continue;
                    } else {
                        acc.push(val);
                    }
                }
                Err(ParseErr::Abort(val)) => {
                    if acc.len() == k {
                        // consumes more `self`, but don't push to `acc`
                        return Err(ParseErr::Abort(Val::Star(acc)));
                    } else {
                        acc.push(val);
                        return Err(ParseErr::Abort(Val::Star(acc)));
                    }
                }
                Err(ParseErr::Invalid(_)) => {
                    *matched = cur;
                    break
                },
            }
        }
        Ok((acc, s))
    }
}

/// Result of parsing
#[derive(Debug)]
pub enum Val<Alphabet> {
    Epsilon,
    Literal(Alphabet),
    Literals(Vec<Alphabet>),
    Concat(Box<Val<Alphabet>>, Box<Val<Alphabet>>),
    Seq(Vec<Val<Alphabet>>),
    Star(Vec<Val<Alphabet>>),
}

impl<Alphabet> Val<Alphabet> {
    pub fn into_vec(self) -> Vec<Alphabet> {
        match self {
            Val::Epsilon => Vec::new(),
            Val::Literal(c) => vec![c],
            Val::Literals(cs) => cs,
            Val::Concat(v1, v2) => {
                let mut r1 = v1.into_vec();
                let mut r2 = v2.into_vec();
                r1.append(&mut r2);
                r1
            }
            Val::Seq(vs) | Val::Star(vs) => {
                let mut res = Vec::new();
                for v in vs {
                    res.append(&mut v.into_vec())
                }
                res
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::re::*;

    #[test]
    fn test1() {
        use RegExp::*;
        // 1(21)*3
        let re: RegExp<_, ()> = Concat(
            Box::new(Literal(1)),
            Box::new(Concat(
                Box::new(Star(Box::new(Concat(
                    Box::new(Literal(2)),
                    Box::new(Literal(1)),
                )))),
                Box::new(Literal(3)),
            )),
        );
        let s = vec![1, 2, 1, 2, 1, 2, 1, 3];
        let (v, _) = re.parse_inf(&s, &BTreeMap::new()).unwrap();
        let k = 2;
        let reduced = v.reduce(k);
        assert!(reduced == vec![1, 2, 1, 2, 1, 3]);
    }

    #[test]
    fn test1_() {
        use RegExp::*;
        // 1(21)*3
        let re: RegExp<_, ()> = Concat(
            Box::new(Literal(1)),
            Box::new(Concat(
                Box::new(Star(Box::new(Concat(
                    Box::new(Literal(2)),
                    Box::new(Literal(1)),
                )))),
                Box::new(Literal(3)),
            )),
        );
        let s = vec![1, 2, 1, 2, 1, 2, 1, 3];
        let k = 2;
        let (v, _) = re
            .parse_k(&s, &BTreeMap::new(), &BTreeMap::new(), k)
            .unwrap();
        let reduced = v.into_vec();
        assert!(reduced == vec![1, 2, 1, 2, 1, 3]);
    }

    #[test]
    fn test2() {
        // (12)*(13)
        let re: RegExp<_, ()> = RegExp::concat(
            RegExp::star(RegExp::concat(RegExp::literal(1), RegExp::literal(2))),
            RegExp::concat(RegExp::literal(1), RegExp::literal(3)),
        );
        let s = vec![1, 2, 1, 2, 1, 2, 1, 3];
        let (v, _) = re.parse_inf(&s, &BTreeMap::new()).unwrap();
        let k = 2;
        let reduced = v.reduce(k);
        assert!(reduced == vec![1, 2, 1, 2, 1, 3]);
    }

    #[test]
    fn test2_() {
        // (12)*(13)
        let re: RegExp<_, ()> = RegExp::concat(
            RegExp::star(RegExp::concat(RegExp::literal(1), RegExp::literal(2))),
            RegExp::concat(RegExp::literal(1), RegExp::literal(3)),
        );
        let s = vec![1, 2, 1, 2, 1, 2, 1, 3];
        let k = 2;
        let (v, _) = re.parse_k(&s, &BTreeMap::new(), k).unwrap();
        let reduced = v.into_vec();
        assert!(reduced == vec![1, 2, 1, 2, 1, 3]);
    }

    #[test]
    fn test3() {
        use RegExp::*;
        let re = RegExp::alter(
            RegExp::concat(
                RegExp::concat(RegExp::concat(Literal(9), Epsilon), Literal(11)),
                RegExp::concat(
                    Literal(13),
                    RegExp::concat(
                        Epsilon,
                        RegExp::concat(
                            Literal(15),
                            RegExp::alter(
                                RegExp::concat(Literal(16), Literal(27)),
                                RegExp::concat(
                                    Literal(17),
                                    RegExp::concat(
                                        Literal(18),
                                        RegExp::concat(
                                            Literal(22),
                                            RegExp::concat(
                                                Var(0),
                                                RegExp::concat(
                                                    Literal(24),
                                                    RegExp::concat(
                                                        Epsilon,
                                                        RegExp::concat(Literal(26), Literal(27)),
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
            RegExp::concat(
                RegExp::concat(
                    RegExp::concat(RegExp::concat(Literal(9), Epsilon), Literal(11)),
                    Literal(12),
                ),
                Literal(27),
            ),
        );
        let re0 = RegExp::alter(
            RegExp::alter(
                RegExp::concat(
                    RegExp::concat(RegExp::Literal(0), RegExp::Literal(2)),
                    RegExp::Literal(8),
                ),
                RegExp::concat(
                    RegExp::concat(RegExp::Literal(0), RegExp::Literal(1)),
                    RegExp::concat(RegExp::Literal(2), RegExp::Literal(8)),
                ),
            ),
            RegExp::concat(
                RegExp::alter(
                    RegExp::concat(
                        RegExp::concat(RegExp::Literal(0), RegExp::Literal(2)),
                        RegExp::Literal(3),
                    ),
                    RegExp::concat(
                        RegExp::concat(RegExp::Literal(0), RegExp::Literal(1)),
                        RegExp::concat(RegExp::Literal(2), RegExp::Literal(3)),
                    ),
                ),
                RegExp::concat(RegExp::Literal(7), RegExp::Literal(8)),
            ),
        );
        let path = vec![9, 11, 13, 15, 17, 18, 22, 0, 2, 3];
        let mut env = BTreeMap::new();
        env.insert(0, re0);
        let res = re.parse_k(&path, &env, 3);
        println!("{:?}", res);
    }

    #[test]
    fn test4() {
        // f = 1(2|f)3
        let re = RegExp::Seq(vec![
            RegExp::literal(1),
            RegExp::alter(RegExp::Literal(2), RegExp::Var(0)),
            RegExp::literal(3),
        ]);
        let s = vec![1, 1, 1, 2, 3, 3, 3];
        let mut env = BTreeMap::new();
        env.insert(0, re.clone());
        let k = 1;
        let (v, _) = re.parse_k(&s, &env, k).unwrap();
        let reduced = v.into_vec();
        assert!(reduced == vec![1, 1, 3, 3]);
    }
}
