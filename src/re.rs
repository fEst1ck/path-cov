//! Regular expressions

use core::panic;
use std::{collections::BTreeMap, fmt::Debug};

/// Regular expressions over alphabet set `Alphabet`, and variable set `Name`
/// a variable refers to an external regular expression
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegExp<Alphabet, Name> {
    Epsilon,
    Var(Name),
    Literal(Alphabet),
    Literals(Vec<Alphabet>),
    Concat(Box<RegExp<Alphabet, Name>>, Box<RegExp<Alphabet, Name>>),
    Seq(Vec<RegExp<Alphabet, Name>>),
    Alter(Box<RegExp<Alphabet, Name>>, Box<RegExp<Alphabet, Name>>),
    Star(Box<RegExp<Alphabet, Name>>),
}

#[derive(Debug)]
pub enum ParseErr<Alphabet> {
    Abort(Val<Alphabet>),
    Invalid,
}

impl<Alphabet: Eq + Clone + Ord + Debug, Name: Eq + Clone + Ord + Debug> RegExp<Alphabet, Name> {
    pub fn first(&self) -> Alphabet {
        match self {
            RegExp::Epsilon => panic!("first: start with epsilon"),
            RegExp::Var(_) => panic!("first: start with var"),
            RegExp::Literal(c) => c.clone(),
            RegExp::Literals(cs) => cs[0].clone(),
            RegExp::Concat(re, _) => re.first(),
            RegExp::Seq(res) => res[0].first(),
            RegExp::Alter(re1, re2) => {
                let f1 = re1.first();
                let f2 = re2.first();
                if f1 == f2 {
                    f1
                } else {
                    panic!("first: may begin with {:?} {:?}", f1, f2)
                }
            },
            RegExp::Star(re) => re.first(),
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

    #[allow(dead_code)]
    pub fn concat(r1: RegExp<Alphabet, Name>, r2: RegExp<Alphabet, Name>) -> Self {
        use RegExp::*;
        match (r1, r2) {
            (Epsilon, r2) => r2,
            (r1, Epsilon) => r1,
            (Literal(l1), Literal(l2)) => Literals(vec![l1, l2]),
            (Literal(l1), Literals(mut l2)) => {
                let mut res = vec![l1];
                res.append(&mut l2);
                Literals(res)
            }
            (Literals(mut l1), Literal(l2)) => {
                l1.push(l2);
                Literals(l1)
            }
            (Literals(mut l1), Literals(mut l2)) => {
                l1.append(&mut l2);
                Literals(l1)
            }
            (Seq(mut es1), Seq(mut es2)) => {
                es1.append(&mut es2);
                Seq(es1)
            }
            (Seq(mut es1), r2) => {
                es1.push(r2);
                Seq(es1)
            }
            (r1, Seq(mut es2)) => {
                let mut es = vec![r1];
                es.append(&mut es2);
                Seq(es)
            }
            (r1, r2) => Seq(vec![r1, r2]),
        }
        //Self::Concat(Box::new(r1), Box::new(r2))
    }

    #[allow(dead_code)]
    pub fn alter(r1: Self, r2: Self) -> Self {
        Self::Alter(Box::new(r1), Box::new(r2))
    }

    pub fn star(r: Self) -> Self {
        Self::Star(Box::new(r))
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
        self._parse_k(s, env, firsts, k, &mut stack)
    }

    pub fn _parse_k<'a>(
        &self,
        s: &'a [Alphabet],
        env: &BTreeMap<Name, RegExp<Alphabet, Name>>,
        firsts: &BTreeMap<Alphabet, Name>,
        k: usize,
        stack: &mut BTreeMap<Name, usize>,
    ) -> Result<(Val<Alphabet>, &'a [Alphabet]), ParseErr<Alphabet>> {
        // println!("_parse_k\nre:{:?}\ns:{:?}", self, s);
        match self {
            RegExp::Epsilon => Ok((Val::Star(Vec::new()), s)),
            RegExp::Var(x) => {
                let re = env
                    .get(x)
                    .expect(&format!("name {:?} doesn't exist in env", x));
                let nested_level = *stack.entry(x.clone()).or_default();
                if nested_level == k {
                    match re._parse_k(s, env, firsts, k, stack) {
                        Ok((_, s)) => Ok((Val::Epsilon, s)),
                        Err(ParseErr::Abort(_)) => Err(ParseErr::Abort(Val::Epsilon)),
                        Err(ParseErr::Invalid) => Err(ParseErr::Invalid),
                    }
                } else {
                    *stack.get_mut(x).unwrap() += 1;
                    let res = re._parse_k(s, env, firsts, k, stack);
                    *stack.get_mut(x).unwrap() -= 1;
                    res
                }
            }
            RegExp::Literal(c) => {
                if s.is_empty() {
                    Err(ParseErr::Abort(Val::Epsilon))
                } else {
                    if c == &s[0] {
                        Ok((Val::Literal(c.clone()), &s[1..]))
                    } else {
                        if let Some(x) = firsts.get(&s[0]) {
                            let re = RegExp::Var(x.clone());
                            re._parse_k(s, env, firsts, k, stack)
                        } else {
                            // println!("expected {:?} found {:?} stack: {:?}", c, &s, &stack);
                            // println!("firsts: {:?}", firsts);
                            Err(ParseErr::Invalid)
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
                                let res = re._parse_k(s, env, firsts, k, stack);
                                match res {
                                    Ok((val, rest_path)) => {
                                        lit_vals.push(val);
                                        rest = rest_path;
                                    },
                                    Err(ParseErr::Abort(v)) => {
                                        lit_vals.push(v);
                                        return Err(ParseErr::Abort(Val::Seq(lit_vals)))
                                    },
                                    Err(ParseErr::Invalid) => {
                                        return Err(ParseErr::Invalid)
                                    }
                                }
                            } else {
                                // println!("expected {:?} found {:?} stack: {:?}", c, &s, &stack);
                                // println!("firsts: {:?}", firsts);
                                return Err(ParseErr::Invalid)
                            }
                        }
                    }
                }
                Ok((Val::Seq(lit_vals), rest))
            }
            RegExp::Concat(r1, r2) => {
                let (v1, s1) = r1._parse_k(s, env, firsts, k, stack)?;
                match r2._parse_k(s1, env, firsts, k, stack) {
                    Ok((v2, s2)) => Ok((Val::Concat(Box::new(v1), Box::new(v2)), s2)),
                    Err(ParseErr::Abort(v2)) => {
                        Err(ParseErr::Abort(Val::Concat(Box::new(v1), Box::new(v2))))
                    }
                    Err(ParseErr::Invalid) => Err(ParseErr::Invalid),
                }
            }
            RegExp::Seq(rs) => {
                let mut vals = Vec::new();
                let mut rest = s;
                for r in rs {
                    match r._parse_k(rest, env, firsts, k, stack) {
                        Ok((v, s)) => {
                            vals.push(v);
                            rest = s;
                        }
                        Err(ParseErr::Abort(v)) => {
                            vals.push(v);
                            return Err(ParseErr::Abort(Val::Seq(vals)));
                        }
                        Err(ParseErr::Invalid) => {
                            return Err(ParseErr::Invalid);
                        }
                    }
                }
                Ok((Val::Seq(vals), rest))
            }
            RegExp::Alter(r1, r2) => match r1._parse_k(s, env, firsts, k, stack) {
                res @ Ok(..) | res @ Err(ParseErr::Abort(..)) => res,
                _ => r2._parse_k(s, env, firsts,  k, stack),
            },
            RegExp::Star(r) => match r.parse_star_k(s, env, firsts, k, stack) {
                Ok((vals, s)) => Ok((Val::Star(vals), s)),
                Err(ParseErr::Abort(val)) => Err(ParseErr::Abort(val)),
                Err(ParseErr::Invalid) => Err(ParseErr::Invalid),
            },
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
        stack: &mut BTreeMap<Name, usize>
    ) -> Result<(Vec<Val<Alphabet>>, &'a [Alphabet]), ParseErr<Alphabet>> {
        let mut acc = Vec::new();
        loop {
            match self._parse_k(s, env, firsts, k, stack) {
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
                Err(ParseErr::Invalid) => break,
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
    #[allow(dead_code)]
    pub fn reduce(self, k: usize) -> Vec<Alphabet> {
        assert!(k != 0);
        match self {
            Val::Literal(c) => vec![c],
            Val::Concat(v1, v2) => {
                let mut r1 = v1.reduce(k);
                let mut r2 = v2.reduce(k);
                r1.append(&mut r2);
                r1
            }
            Val::Star(vs) => {
                let mut res = Vec::new();
                let mut counter = 0;
                for v in vs {
                    if counter >= k {
                        break;
                    } else {
                        res.append(&mut v.reduce(k))
                    }
                    counter += 1;
                }
                res
            }
            Val::Epsilon => todo!(),
            Val::Literals(_) => todo!(),
            Val::Seq(_) => todo!(),
        }
    }

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
        let (v, _) = re.parse_k(&s, &BTreeMap::new(), &BTreeMap::new(), k).unwrap();
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
        let re = RegExp::Seq(vec![RegExp::literal(1), RegExp::alter(RegExp::Literal(2), RegExp::Var(0)), RegExp::literal(3)]);
        let s = vec![1, 1, 1, 2, 3, 3, 3];
        let mut env = BTreeMap::new();
        env.insert(0, re.clone());
        let k = 1;
        let (v, _) = re.parse_k(&s, &env, k).unwrap();
        let reduced = v.into_vec();
        assert!(reduced == vec![1, 1, 3, 3]);
    }
}
