//! Regular expressions

use std::{collections::BTreeMap, fmt::Display, fmt::Debug};

/// Regular expressions over alphabet set `Alphabet`, and variable set `Name`
/// a variable refers to an external regular expression
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegExp<Alphabet, Name> {
    Epsilon,
    Var(Name),
    Literal(Alphabet),
    Concat(Box<RegExp<Alphabet, Name>>, Box<RegExp<Alphabet, Name>>),
    Alter(Box<RegExp<Alphabet, Name>>, Box<RegExp<Alphabet, Name>>),
    Star(Box<RegExp<Alphabet, Name>>),
}

#[derive(Debug)]
pub enum ParseErr<Alphabet> {
    Abort(Val<Alphabet>),
    Invalid,
}

impl<Alphabet: Eq + Clone + Debug, Name: Eq + Clone + Ord + Debug> RegExp<Alphabet, Name> {
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
        Self::Concat(Box::new(r1), Box::new(r2))
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
        }
    }

    pub fn parse_k<'a>(
        &self,
        s: &'a [Alphabet],
        env: &BTreeMap<Name, RegExp<Alphabet, Name>>,
        k: usize,
    ) -> Result<(Val<Alphabet>, &'a [Alphabet]), ParseErr<Alphabet>> {
        match self {
            RegExp::Epsilon => Ok((Val::Star(Vec::new()), s)),
            RegExp::Var(x) => {
                let re = env.get(x).expect(&format!{"name {:?} doesn't exist in env",x});
                re.parse_k(s, env, k)
            }
            RegExp::Literal(c) => {
                if s.is_empty() {
                    Err(ParseErr::Abort(Val::Star(Vec::new())))
                } else {
                    if c == &s[0] {
                        Ok((Val::Literal(c.clone()), &s[1..]))
                    } else {
                        // println!("expected {:?} found {:?}", c, &s[0]);
                        Err(ParseErr::Invalid)
                    }
                }
            }
            RegExp::Concat(r1, r2) => {
                let (v1, s1) = r1.parse_k(s, env, k)?;
                match r2.parse_k(s1, env, k) {
                    Ok((v2, s2)) => Ok((Val::Concat(Box::new(v1), Box::new(v2)), s2)),
                    Err(ParseErr::Abort(v2)) => Err(ParseErr::Abort(Val::Concat(Box::new(v1), Box::new(v2)))),
                    Err(ParseErr::Invalid) => Err(ParseErr::Invalid),
                }
            }
            RegExp::Alter(r1, r2) => match r1.parse_k(s, env, k) {
                res @ Ok(..) | res @ Err(ParseErr::Abort(..)) => res,
                _ => r2.parse_k(s, env, k),
            },
            RegExp::Star(r) => {
                match r.parse_star_k(s, env, k) {
                    Ok((vals, s)) => Ok((Val::Star(vals), s)),
                    Err(ParseErr::Abort(val)) => Err(ParseErr::Abort(val)),
                    Err(ParseErr::Invalid) => Err(ParseErr::Invalid),
                }
            }
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
        k: usize,
    ) -> Result<(Vec<Val<Alphabet>>, &'a [Alphabet]), ParseErr<Alphabet>> {
        let mut acc = Vec::new();
        loop {
            match self.parse_k(s, env, k) {
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
    Literal(Alphabet),
    Concat(Box<Val<Alphabet>>, Box<Val<Alphabet>>),
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
        }
    }

    pub fn into_vec(self) -> Vec<Alphabet> {
        match self {
            Val::Literal(c) => vec![c],
            Val::Concat(v1, v2) => {
                let mut r1 = v1.into_vec();
                let mut r2 = v2.into_vec();
                r1.append(&mut r2);
                r1
            }
            Val::Star(vs) => {
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
        let (v, _) = re.parse_k(&s, &BTreeMap::new(), k).unwrap();
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
        let re = RegExp::alter(RegExp::concat(RegExp::concat(RegExp::concat(Literal(9), Epsilon), Literal(11)), RegExp::concat(Literal(13), RegExp::concat(Epsilon, RegExp::concat(Literal(15), RegExp::alter(RegExp::concat(Literal(16), Literal(27)), RegExp::concat(Literal(17), RegExp::concat(Literal(18), RegExp::concat(Literal(22), RegExp::concat(Var(0), RegExp::concat(Literal(24), RegExp::concat(Epsilon, RegExp::concat(Literal(26), Literal(27))))))))))))), RegExp::concat(RegExp::concat(RegExp::concat(RegExp::concat(Literal(9), Epsilon), Literal(11)), Literal(12)), Literal(27)));
        let re0 =  RegExp::alter(RegExp::alter(RegExp::concat(RegExp::concat(RegExp::Literal(0), RegExp::Literal(2)), RegExp::Literal(8)), RegExp::concat(RegExp::concat(RegExp::Literal(0), RegExp::Literal(1)), RegExp::concat(RegExp::Literal(2), RegExp::Literal(8)))), RegExp::concat(RegExp::alter(RegExp::concat(RegExp::concat(RegExp::Literal(0), RegExp::Literal(2)), RegExp::Literal(3)), RegExp::concat(RegExp::concat(RegExp::Literal(0), RegExp::Literal(1)), RegExp::concat(RegExp::Literal(2), RegExp::Literal(3)))), RegExp::concat(RegExp::Literal(7), RegExp::Literal(8))));
        let path = vec![9, 11, 13, 15, 17, 18, 22, 0, 2, 3, 4];
        let mut env = BTreeMap::new();
        env.insert(0, re0);
        let res = re.parse_k(&path, &env, 3);
        println!("{:?}", res);
    }
}
